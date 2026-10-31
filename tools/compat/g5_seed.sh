#!/bin/bash
# PHP 그누보드 화면 요청으로 데이터를 만든다 (관리자 로그인 쿠키 pj.txt)
P=http://127.0.0.1:8081; CK="-c pj.txt -b pj.txt -H Origin:$P -H Referer:$P/bbs/write.php"
token(){ curl -s $CK -X POST -d "bo_table=$1" $P/bbs/write_token.php | python3 -c "import sys,json;print(json.load(sys.stdin)['token'])"; }
# write BO SUBJECT CONTENT [extra curl -F args...]
write(){ bo=$1; subj=$2; body=$3; shift 3; t=$(token $bo); uid=$(date +%s%N | cut -c1-16)
  curl -s $CK -o /dev/null -w "%{redirect_url}\n" -X POST $P/bbs/write_update.php -F "w=" -F "bo_table=$bo" -F "wr_id=0" -F "uid=$uid" -F "token=$t" --form-string "wr_subject=$subj" --form-string "wr_content=$body" "$@"; }
reply(){ bo=$1; parent=$2; subj=$3; shift 3; t=$(token $bo); uid=$(date +%s%N | cut -c1-16)
  curl -s $CK -o /dev/null -w "%{redirect_url}\n" -X POST $P/bbs/write_update.php -F "w=r" -F "bo_table=$bo" -F "wr_id=$parent" -F "uid=$uid" -F "token=$t" -F "wr_subject=$subj" -F "wr_content=답글 내용" "$@"; }
# comment BO WR_ID CONTENT [comment_id] [secret]
comment(){ bo=$1; wr=$2; body=$3; cid=$4; sec=$5
  curl -s $CK "$P/bbs/board.php?bo_table=$bo&wr_id=$wr" -o /dev/null; t=$(curl -s $CK "$P/bbs/ajax.comment_token.php" | python3 -c "import sys,json;print(json.load(sys.stdin)['token'])")
  curl -s $CK -o /dev/null -w "%{http_code} " -X POST $P/bbs/write_comment_update.php --data-urlencode "w=c" --data-urlencode "bo_table=$bo" --data-urlencode "wr_id=$wr" --data-urlencode "comment_id=$cid" --data-urlencode "token=$t" --data-urlencode "wr_content=$body" --data-urlencode "wr_secret=$sec"; }
