# 03. 구조 (Architecture)

## 1. 전체 구성

```text
[브라우저] ──HTTPS──> [Rpanel 리버스 프록시 / Nginx]
                           │
              ┌────────────┴────────────┐
              │                         │
       /api/*, /ws              /, /assets/* (React 빌드 정적 파일)
              │                         │
      [bp-server (Axum)]  ◄────  SSR 없음, SPA + SEO용 프리렌더(선택)
        │      │      │
   MariaDB   Redis   data/ (파일 스토리지, g5와 동일 경로)
  (g5_* +bp_*)
```

## 2. 저장소 구조 (모노레포)

```text
beripickboard/
├── doc/                      # 이 문서들
├── server/                   # Rust workspace
│   ├── Cargo.toml
│   ├── crates/
│   │   ├── bp-server/        # Axum 진입점, 라우터, 미들웨어
│   │   ├── bp-core/          # 도메인 로직 (회원, 게시판, 포인트, 쇼핑몰)
│   │   ├── bp-g5compat/      # ★ g5 호환 계층: 테이블 매핑, 해시, wr_reply 계산, 파일 경로
│   │   ├── bp-shop/          # 영카트 (상품, 장바구니, 주문, 쿠폰)
│   │   ├── bp-pg/            # PG 어댑터 (KCP, 이니시스, 토스, 나이스 …)
│   │   ├── bp-eyoom/         # 이윰보드 확장 테이블 매핑
│   │   ├── bp-license/       # 라이센스 발급/검증
│   │   └── bp-migrate/       # CLI: 스키마 최신화, InnoDB/utf8mb4 변환, 검증
│   └── migrations/           # bp_* 테이블 마이그레이션 (sqlx)
├── web/                      # React 19 + Vite + Tailwind 4
│   ├── src/
│   │   ├── app/              # 라우팅, 전역 상태
│   │   ├── api/              # API 클라이언트 (타입 자동 생성)
│   │   ├── themes/           # 테마 = g5 theme/{name} 대응
│   │   │   └── basic/
│   │   │       ├── layout/   # head/tail 대응
│   │   │       └── skin/     # board/, latest/, member/, shop/ … (g5 skin 폴더명 그대로)
│   │   ├── widgets/          # 이윰 EB 콘텐츠/슬라이더/최신글 대응
│   │   └── admin/            # 관리자 (adm/ 대응)
│   └── vite.config.ts
├── deploy/                   # Rpanel/systemd/nginx 템플릿
└── docker-compose.yml        # 로컬 MariaDB + Redis
```

## 3. 백엔드 (Axum)
- 런타임: tokio, 프레임워크: axum, DB: **sqlx (MySQL 드라이버, MariaDB 호환)**, 캐시: redis(deadpool-redis)
- 계층: `router → handler → service(bp-core) → repository(bp-g5compat)`
- 설정: `.env` + `config.toml`. 테이블 접두사 `G5_TABLE_PREFIX` 설정 지원 (기본 `g5_`)
- 동적 테이블 `g5_write_{bo_table}` 는 컴파일 타임 검사(`query!`)가 불가하므로:
  - `bo_table`은 반드시 `g5_board`에 존재하는 값만 허용 (Redis 캐시된 화이트리스트)
  - 정규식 `^[A-Za-z0-9_]{1,20}$` 재검증 후 백틱으로 감싸 쿼리 조립
  - 컬럼 매핑은 `WriteRow` 구조체 하나로 통일 (`adm/sql_write.sql` 기준 40개 컬럼)

## 4. DB 설계 — g5 스키마 호환

### 4.1 그대로 사용하는 테이블 (g5 5.6.44 기준)
| 영역 | 테이블 |
|---|---|
| 기본 | g5_config, g5_migrations, g5_uniqid, g5_autosave |
| 회원 | g5_member, g5_member_auto_login, g5_member_social_profiles, g5_member_cert_history, g5_cert_history, g5_login, g5_auth |
| 게시판 | g5_group, g5_group_member, g5_board, g5_write_{bo_table}, g5_board_file, g5_board_good, g5_board_new |
| 부가 | g5_point, g5_memo, g5_scrap, g5_poll, g5_poll_etc, g5_popular, g5_visit, g5_visit_sum, g5_mail, g5_abuse_rate |
| 콘텐츠 | g5_content, g5_faq, g5_faq_master, g5_qa_config, g5_qa_content, g5_new_win, g5_menu |
| 영카트 | g5_shop_default, g5_shop_category, g5_shop_item, g5_shop_item_option, g5_shop_item_use, g5_shop_item_qa, g5_shop_item_relation, g5_shop_item_stocksms, g5_shop_wish, g5_shop_cart, g5_shop_order, g5_shop_order_data, g5_shop_order_address, g5_shop_order_delete, g5_shop_order_cancel_log, g5_shop_order_access, g5_shop_order_post_log, g5_shop_coupon, g5_shop_coupon_log, g5_shop_coupon_zone, g5_shop_event, g5_shop_event_item, g5_shop_banner, g5_shop_personalpay, g5_shop_sendcost |
| PG 로그 | g5_shop_inicis_log, g5_shop_inicis_pay, g5_shop_inicis_pay_event, g5_shop_kcp_noti, g5_shop_lg_noti, g5_shop_pg_approval_log |
| 이윰보드 | `g5_eyoom_*` 계열 — > TODO(확인): 실제 설치본 `SHOW TABLES LIKE '%eyoom%'` 결과로 목록 확정 후 `bp-eyoom`에 매핑 |

### 4.2 신규 테이블 (`bp_` 접두사, g5 테이블과 분리)
| 테이블 | 용도 |
|---|---|
| bp_license | 라이센스 키, 도메인, 만료일, 서명 |
| bp_widget / bp_widget_slot | 위젯 인스턴스와 배치(테마 영역) |
| bp_refresh_token | JWT 리프레시 토큰 (Redis 장애 대비 영속본) |
| bp_job | 백그라운드 작업 (메일, 썸네일, 정산 집계) |
| bp_settlement / bp_pg_fee | 정산 집계, PG 수수료율 (→ 10 문서) |
| bp_schema_version | beripickboard 자체 스키마 버전 |

### 4.3 g5 데이터 규칙 중 반드시 지켜야 하는 것
- **게시글 정렬:** `wr_num`은 음수 감소값(새 글일수록 작음), 답글은 같은 `wr_num`에 `wr_reply`가 `A`, `AA`, `AB`… 문자열로 붙음. 목록은 `ORDER BY wr_num, wr_reply`.
- **댓글:** 같은 테이블에 `wr_is_comment=1`, `wr_parent`=원글 id, `wr_comment`(댓글 순번), `wr_comment_reply`(대댓글 문자열). 원글의 `wr_comment`는 댓글 수.
- **원글:** `wr_parent = wr_id`, `wr_is_comment = 0`.
- **wr_option:** `set('html1','html2','secret','mail')` — html1/html2는 HTML 사용 여부, secret은 비밀글.
- **카운터 동기화:** `g5_board.bo_count_write`, `bo_count_comment`, `g5_member.mb_point`(= `g5_point` 합계), `g5_board_new`(새글) 등 비정규화 값을 PHP와 동일하게 갱신.
- **포인트:** 모든 증감은 `g5_point` 행 삽입 + `mb_point` 갱신을 한 트랜잭션으로. `po_rel_table/po_rel_id/po_rel_action` 으로 중복 지급 방지.
- **회원 레벨:** `mb_level` 1~10, 게시판 권한 `bo_*_level`과 비교.
- **날짜:** `datetime` 기본값 `0000-00-00 00:00:00` 존재 → Rust에서는 `Option<NaiveDateTime>`으로 읽고 0일자는 `None` 처리. sqlx 연결 시 `sql_mode`에서 `NO_ZERO_DATE` 제거 필요.
- **인코딩:** 원본은 `utf8`(3바이트). 마이그레이션 시 `utf8mb4`로 변환 권장(이모지).

### 4.4 엔진 변환
원본 g5 테이블은 **MyISAM**. 주문·포인트 트랜잭션을 위해 `bp-migrate`가 `ALTER TABLE … ENGINE=InnoDB` 를 수행한다. 공존 기간에도 PHP는 InnoDB에서 그대로 동작한다.

## 5. 인증
- **비밀번호 검증 (bp-g5compat::password)**
  | 저장 형식 | 판별 | 처리 |
  |---|---|---|
  | `sha256:12000:{salt}:{hash}` | `:` 4분할 | PBKDF2-HMAC-SHA256, salt는 base64 문자열 자체를 salt로 사용, 키 길이 24바이트 |
  | `*` + 40 hex | MySQL `PASSWORD()` | `'*' + UPPER(HEX(SHA1(SHA1(pw))))` |
  | 16 hex | `OLD_PASSWORD()` | 검증 후 즉시 PBKDF2로 재해시 |
- 공존 기간에는 **새 해시도 g5 PBKDF2 형식으로 저장**(PHP 쪽 로그인 유지). 약한 해시의 로그인 시 재저장은 `G5_REHASH_ON_LOGIN=true` 일 때만(PHP가 `create_hash` 설정일 때). 완전 전환 후 Argon2id 로 승급 옵션.
- 세션: Access JWT(HS256, 15분, 메모리 보관) + Refresh 토큰(HttpOnly 쿠키 `bp_rt`, `bp_refresh_token` 테이블에 SHA-256 해시만 저장). 회전 방식이며 이미 쓴 토큰이 다시 오면 같은 패밀리 전체 폐기. PHP 세션과는 공유하지 않음.
- 로그인 규칙은 `bbs/login_check.php` 와 동일: 아이디·비번 trim, 차단일(`mb_intercept_date`)·탈퇴일(`mb_leave_date`) Ymd 비교, `cf_use_email_certify` 시 메일인증 확인, 최고관리자(`cf_admin`) 자동로그인 금지.
- 오늘 첫 접속 시(common.php) `cf_login_point` 지급(`@login` / mb_id / Y-m-d, PHP와 같은 `GET_LOCK('g5pt_'+md5)` 사용) + `mb_today_login`, `mb_login_ip` 갱신.
- 로그인 실패 제한: 같은 IP+아이디 10분 10회 (단일 인스턴스 메모리, 다중 인스턴스 시 Redis 로 교체).

## 6. 파일 스토리지
- `DATA_PATH`를 기존 g5 `data/` 디렉토리로 지정 → 첨부파일 그대로 사용.
- 첨부파일: `data/file/{bo_table}/{bf_file}`, 메타는 `g5_board_file`(`bf_storage` 컬럼으로 외부 스토리지 확장 대비).
- 에디터 이미지: `data/editor/{yymm}/…` — 본문 HTML 내 절대경로를 그대로 서빙.
- 상품 이미지: `data/item/{it_id}/…`, 회원 아이콘/이미지: `data/member/`, `data/member_image/`.
- 다운로드는 반드시 API(`/api/v1/boards/{bo_table}/posts/{wr_id}/files/{bf_no}?t=`)를 거쳐 권한·포인트 차감 후 스트리밍. `t` 는 글 보기 응답에만 들어 있는 HMAC 서명 토큰(1시간, 게시판·글·열람자 고정)이라 `<a href>` 로 바로 받을 수 있고, PHP 의 "보기 세션이 있어야 다운로드" 규칙과 같다.
- PHP 세션 플래그 대응: 조회수·읽기/다운로드 포인트 중복 방지는 서버 메모리 캐시(열람자=회원 아이디 또는 비회원 IP, 3시간), 비밀글 열람 허가는 서명 토큰(`x-secret-grant`, 3시간)으로 대신한다.
- 포인트는 `insert_point` / `insert_use_point`(FIFO 차감) / `get_point_sum`(소멸 처리)을 그대로 옮겼고 PHP 와 같은 named lock 을 쓴다.

## 7. 공존 운영 (PHP ↔ beripickboard)
1. 단계 A: 읽기 전용 — 새 CMS는 조회만, 쓰기는 PHP.
2. 단계 B: 게시판·회원 쓰기 전환 — PHP는 관리자만.
3. 단계 C: 쇼핑몰 전환 — PG 콜백 URL 변경.
4. 단계 D: PHP 종료, InnoDB/utf8mb4 최종 정리, 해시 승급.
- 캐시 주의: PHP가 쓴 데이터를 감지할 수 없으므로 공존 기간 Redis 캐시 TTL은 짧게(≤60초) 유지.

## 8. 프론트엔드
- React 19 함수형 컴포넌트, React Router, TanStack Query(서버 상태), Zustand(UI 상태)
- 테마 로딩: `g5_config.cf_theme` 값 → `web/src/themes/{cf_theme}` 동적 import
- 스킨 매핑: `g5_board.bo_skin` 값(예: `basic`, `gallery`, `theme/basic`) → 스킨 레지스트리에서 컴포넌트 선택, 없으면 `basic` 폴백
- 관리자: `/adm` 하위, 메뉴 코드는 g5와 동일(`100` 환경설정, `200` 회원, `300` 게시판, `400` 쇼핑몰, `500` 쇼핑몰현황, `900` SMS) → `g5_auth.au_menu` 권한 그대로 적용
