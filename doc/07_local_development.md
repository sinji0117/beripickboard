# 07. 로컬 개발 환경 구축

## 1. 필수 설치 (Windows 기준, macOS/Linux 동일)
| 도구 | 버전 | 설치 |
|---|---|---|
| Rust | stable 최신 | https://rustup.rs → `rustup component add clippy rustfmt` |
| Node.js | 22 LTS 이상 | + `corepack enable` (pnpm) |
| Docker Desktop | 최신 | MariaDB·Redis 컨테이너용 |
| sqlx-cli | 최신 | `cargo install sqlx-cli --no-default-features --features mysql,rustls` |

## 2. DB · Redis 기동
```yaml
# docker-compose.yml
services:
  mariadb:
    image: mariadb:11.4
    environment:
      MARIADB_ROOT_PASSWORD: root
      MARIADB_DATABASE: beripick
      MARIADB_USER: beripick
      MARIADB_PASSWORD: beripick
    command: --character-set-server=utf8mb4 --collation-server=utf8mb4_unicode_ci --sql-mode=""
    ports: ["3306:3306"]
    volumes: ["./.local/mysql:/var/lib/mysql"]
  redis:
    image: redis:7
    ports: ["6379:6379"]
```
`--sql-mode=""` 는 g5의 `0000-00-00 00:00:00` 기본값 호환을 위해 필요하다.

## 3. 기존 그누보드/영카트 DB 붙이기
1. 운영 서버에서 덤프: `mysqldump --single-transaction --default-character-set=utf8 -u USER -p DBNAME > g5.sql`
2. 로컬 반영: `docker exec -i <mariadb> mariadb -uroot -proot beripick < g5.sql`
3. `data/` 폴더를 `D:\git\beripickboard\.local\data` 로 복사 (최소 `file/`, `editor/`, `item/`)
4. 스키마 점검: `cargo run -p bp-migrate -- check` (누락 컬럼/구버전 감지)
5. 신규 테이블 생성: `cargo run -p bp-migrate -- up`

새로 시작하는 경우 `gnuboard5.sql` + `gnuboard5shop.sql`(그누보드5 저장소 `install/`)을 먼저 넣는다.

## 4. .env
루트의 `.env.example` 을 `server/.env` 로 복사한다 (`bp-server`, `bp-migrate` 는 실행 위치의 `.env` 를 읽음).
```dotenv
APP_ENV=local
APP_ADDR=127.0.0.1:8080
DATABASE_URL=mysql://beripick:beripick@127.0.0.1:3306/beripick
G5_TABLE_PREFIX=g5_
REDIS_URL=redis://127.0.0.1:6379
DATA_PATH=../.local/data
JWT_SECRET=change-me-32bytes-min
LICENSE_PUBLIC_KEY=   # 개발은 비워두면 dev 모드
CORS_ORIGIN=http://localhost:5173
```
`.env`는 커밋 금지, `.env.example`만 커밋.

> 참고: `@vitejs/plugin-react` 6.x 는 Vite 8 전용이므로 Vite 7 에서는 5.x 를 쓴다.

## 5. 개발 서버
```bash
# 백엔드
cd server && cargo run -p bp-server        # http://127.0.0.1:8080/api/v1/health
# 프론트
cd web && pnpm install && pnpm dev          # http://localhost:5173 (/api → 8080 프록시)
```
