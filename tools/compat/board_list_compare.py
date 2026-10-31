#!/usr/bin/env python3
"""그누보드 PHP 와 beripickboard 의 게시판 목록을 같은 DB 에서 비교한다 (doc/14_testing.md §2).

사용법:
  PHP_URL=http://127.0.0.1:8081 API_URL=http://127.0.0.1:8080/api/v1 BO_TABLE=free \
    python3 tools/compat/board_list_compare.py
PHP 쪽은 basic 스킨 기준으로 목록 HTML 을 읽는다. 둘 다 비회원으로 조회한다.
"""
import json, os, re, sys, urllib.request

P = os.environ.get('PHP_URL', 'http://127.0.0.1:8081')
B = os.environ.get('API_URL', 'http://127.0.0.1:8080/api/v1')
BO = os.environ.get('BO_TABLE', 'free')
CASES = os.environ.get('CASES', 'page=1|page=2|page=3|sfl=wr_subject&stx=%EA%B8%80|sfl=wr_content&stx=%EB%8C%93%EA%B8%80|sst=wr_hit&sod=desc|sst=wr_datetime&sod=asc').split('|')


def php(qs):
    html = urllib.request.urlopen(f'{P}/bbs/board.php?bo_table={BO}&{qs}').read().decode()
    body = html.split('id="fboardlist"')[1].split('</form>')[0]
    out = []
    for cls, row in re.findall(r'<tr class="([^"]*)">(.*?)</tr>', body, re.S):
        m = re.search(r'wr_id=(\d+)', row)
        n = re.search(r'<td class="td_num2">(.*?)</td>', row, re.S)
        if m:
            num = re.sub(r'<[^>]+>|\s', '', n.group(1)) if n else ''
            out.append((int(m.group(1)), num))
    return out


def api(qs):
    d = json.load(urllib.request.urlopen(f'{B}/boards/{BO}/posts?{qs}'))['data']
    return [(i['wrId'], '공지' if i['isNotice'] else str(i['num'])) for i in d['items']]


ok = True
for qs in CASES:
    a, b = php(qs), api(qs)
    ok &= a == b
    print('OK  ' if a == b else 'DIFF', qs, '\n   php:', a, '\n   api:', b)
sys.exit(0 if ok else 1)
