# 11. Rpanel에서 새 앱 설정

> TODO(확인): Rpanel의 실제 메뉴명·화면 구성을 확인해 단계별 캡처로 교체한다. 아래는 일반적인 패널 기준 절차다.

## 1. 앱 생성
| 항목 | 값 |
|---|---|
| 앱 이름 | `beripickboard` (스테이징은 `beripickboard-stg`) |
| 런타임 | 사용자 정의 바이너리 (Rust 빌드 결과 실행) |
| 실행 명령 | `/srv/beripickboard/current/bp-server` |
| 작업 디렉토리 | `/srv/beripickboard/current` |
| 내부 포트 | `8080` (외부 비공개) |
| 실행 사용자 | 전용 계정 `beripick` (root 금지) |

## 2. 디렉토리
```text
/srv/beripickboard/
├── releases/2026xxxx-HHMM/   # 배포본 (bp-server, web/dist)
├── current -> releases/…     # 심볼릭 링크
├── shared/.env               # 환경변수 (권한 600)
└── shared/data/              # g5 data/ (첨부·에디터·상품 이미지) — 배포와 무관하게 유지
```

## 3. 리버스 프록시
```nginx
server {
  server_name example.com;
  client_max_body_size 50m;              # 첨부 최대 용량에 맞춤
  root /srv/beripickboard/current/web/dist;

  location /api/ { proxy_pass http://127.0.0.1:8080; proxy_set_header Host $host;
                   proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
                   proxy_set_header X-Forwarded-Proto $scheme; }
  location /ws   { proxy_pass http://127.0.0.1:8080; proxy_http_version 1.1;
                   proxy_set_header Upgrade $http_upgrade; proxy_set_header Connection "upgrade"; }
  location /data/editor/ { alias /srv/beripickboard/shared/data/editor/; }   # 공개 이미지만 직접 서빙
  location /data/item/   { alias /srv/beripickboard/shared/data/item/; }
  location /data/file/   { return 404; }  # 첨부파일은 반드시 API 경유
  location / { try_files $uri /index.html; }
}
```
기존 PHP 사이트 URL 호환이 필요하면 `/bbs/board.php?bo_table=x&wr_id=y` → `/b/x/y` 301 리다이렉트 규칙을 추가한다(→ 13 문서).

## 4. SSL
- Let's Encrypt 자동 발급/갱신 활성화, HTTP→HTTPS 강제
- HSTS는 첫 배포 안정화 후 활성화
