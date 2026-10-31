# beripickboard (베리픽보드)

그누보드5 · 영카트5 · 이윰보드4의 구조와 DB를 그대로 계승하는 Rust(Axum) + React 19 CMS.
기존 사이트의 DB와 `data/` 폴더를 그대로 연결해 이전하는 것을 목표로 한다.

- 문서: [doc/02_table_of_contents.md](doc/02_table_of_contents.md)
- 현재 버전: v0.0.1

## 구조
```text
server/   Rust workspace — bp-server(API), bp-g5compat(g5 호환), bp-migrate(DB 점검/마이그레이션)
web/      React 19 + Vite + Tailwind CSS 4
doc/      설계 문서 01~14
```

## 빠른 시작
```bash
docker compose up -d                       # MariaDB + Redis
cp .env.example server/.env
cd server
cargo run -p bp-migrate -- check           # 기존 g5 DB 점검 (읽기 전용)
cargo run -p bp-migrate -- up              # bp_* 테이블 생성
cargo run -p bp-server                     # http://127.0.0.1:8080/api/v1/health
cd ../web && pnpm install && pnpm dev      # http://localhost:5173
```
자세한 내용은 [doc/07_local_development.md](doc/07_local_development.md).
