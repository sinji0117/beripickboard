# 그누보드 PHP 호환 비교 도구

같은 DB 를 그누보드 PHP 와 beripickboard 가 함께 바라보게 하고, **PHP 화면으로 만든 데이터**를 두 쪽에서 읽어 결과를 비교한다.

## 준비 (로컬)
```bash
git clone --depth 1 https://github.com/gnuboard/gnuboard5 /tmp/g5run
mkdir -p /tmp/g5run/data && chmod 777 /tmp/g5run/data
cd /tmp/g5run && php -S 127.0.0.1:8081 &          # PHP 8.x + mysqli
# 브라우저에서 http://127.0.0.1:8081/install 로 설치 (DB 이름 예: g5php)
DATABASE_URL=mysql://user:pass@127.0.0.1:3306/g5php cargo run -p bp-migrate -- up
DATABASE_URL=mysql://user:pass@127.0.0.1:3306/g5php DATA_PATH=/tmp/g5run/data cargo run -p bp-server
```

## 데이터 만들기 — `g5_seed.sh`
PHP 의 실제 처리(write_update.php, write_comment_update.php)를 거쳐 글·답글·댓글·첨부를 만든다.
관리자 로그인 쿠키(`pj.txt`)가 필요하다.
```bash
source tools/compat/g5_seed.sh
write free "제목" "내용" -F "html=html1"          # 글
reply free 2 "답글 제목"                         # 답글
comment free 2 "댓글" [부모댓글id] [secret]      # 댓글 / 대댓글 / 비밀댓글
```

## 비교
- `board_list_compare.py` — 목록 순서·글 번호(공지 포함)·검색·정렬 비교
- 포인트: 회원 둘을 같은 상태로 만들고 하나는 PHP, 하나는 API 로 같은 동작을 한 뒤 `g5_point` 행을 비교 (v0.0.4 에서 읽기·다운로드·소멸 포인트 일치 확인)
