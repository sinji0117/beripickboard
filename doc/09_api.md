# 09. API 명세 (Axum REST / WebSocket)

## 1. 공통
- Base: `/api/v1`
- 형식: JSON (UTF-8), 필드 camelCase. g5 컬럼은 접두사만 유지 (`wrSubject`, `mbId`)
- 인증: `Authorization: Bearer <access_jwt>` / 리프레시는 HttpOnly 쿠키 `bp_rt`
- 페이지: `?page=1&rows=15` (기본 rows = `bo_page_rows`)
- 응답
```json
{ "data": { }, "meta": { "page": 1, "rows": 15, "total": 230 } }
```
- 에러
```json
{ "error": { "code": "BOARD_NO_PERMISSION", "message": "글을 읽을 권한이 없습니다.", "status": 403 } }
```

## 2. 인증 토큰
| 토큰 | 수명 | 저장 | 클레임 |
|---|---|---|---|
| Access (JWT HS256) | 15분 | 메모리 | `sub`=mb_id, `lvl`=mb_level, `adm`=관리자 여부, `exp` |
| Refresh (opaque, 32바이트 hex) | 자동로그인 14일 / 일반 1일(세션 쿠키) | bp_refresh_token(해시), 쿠키 `bp_rt` (HttpOnly, SameSite=Lax, Path=/api/v1/auth, 운영 Secure) | 회전 방식, 재사용 감지 시 패밀리 전체 폐기 |

## 3. 엔드포인트 (v0.1.0 범위)

### 인증 · 회원
| 메서드 | 경로 | 설명 |
|---|---|---|
| POST | /auth/login | `{mbId, password, autoLogin}` → `{data:{accessToken, expiresIn, member}}` + `Set-Cookie: bp_rt` ✅ |
| POST | /auth/refresh | 쿠키 `bp_rt` 로 재발급·회전, 응답 형식은 login 과 동일 ✅ |
| POST | /auth/logout | 같은 패밀리 리프레시 전체 폐기, 204 ✅ |
| POST | /members | 회원가입 (`cf_register_level`, `cf_register_point` 적용) |
| GET | /members/me | 내 정보 `{mbId, mbNick, mbName, mbEmail, mbLevel, mbPoint, isAdmin}` ✅ |
| PATCH | /members/me | 정보 수정 |
| DELETE | /members/me | 탈퇴 (`mb_leave_date` 기록) |
| POST | /members/password-lost | 비밀번호 찾기 |
| GET | /members/me/points | 포인트 내역 (g5_point) |
| GET/POST | /members/me/memos | 쪽지 |
| GET/POST/DELETE | /members/me/scraps | 스크랩 |

### 게시판
| 메서드 | 경로 | 설명 |
|---|---|---|
| GET | /boards | 목록 권한(bo_list_level)이 있는 게시판 ✅ |
| GET | /boards/{boTable}/posts | 목록 `?page&sca&sfl&stx&sop&spt&sst&sod` (g5 파라미터명 그대로). 응답에 게시판 설정 `board` 포함 ✅ |
| GET | /boards/{boTable}/posts/{wrId} | 보기 + 첨부 + 댓글 + 이전/다음 + 권한. 조회수·읽기 포인트 처리. 비밀글은 헤더 `x-secret-grant` 필요 ✅ |
| POST | /boards/{boTable}/posts/{wrId}/unlock | `{password, commentId?}` 비밀글/비밀댓글 비밀번호 확인 → `{grant}` (3시간) ✅ |
| POST | /boards/{boTable}/posts | 쓰기 (multipart) |
| POST | /boards/{boTable}/posts/{wrId}/replies | 답글 |
| PATCH/DELETE | /boards/{boTable}/posts/{wrId} | 수정/삭제 (비회원은 `wr_password`) |
| GET/POST | /boards/{boTable}/posts/{wrId}/comments | 댓글 / 대댓글(`parentCommentId`) |
| POST | /boards/{boTable}/posts/{wrId}/good · /nogood | 추천/비추천 |
| GET | /boards/{boTable}/posts/{wrId}/files/{bfNo}?t= | 다운로드 (권한·포인트·횟수). `t` 는 보기 응답이 주는 1시간짜리 서명 토큰 ✅ |
| GET | /boards/{boTable}/posts/{wrId}/files/{bfNo}/inline?t= | 본문 이미지 표시 (포인트·횟수 없음) ✅ |
| GET | /new | 새글 |
| GET | /search | 전체 검색 |

### 콘텐츠 · 위젯
| 메서드 | 경로 |
|---|---|
| GET | /site — `{title, theme}` (g5_config) ✅ |
| GET | /contents/{coId}, /faq, /qa, /menus, /popups, /polls/{poId}, /widgets?slot= |

### 쇼핑몰
| 메서드 | 경로 | 설명 |
|---|---|---|
| GET | /shop/categories | 분류 트리 |
| GET | /shop/items?caId=&type=1 | 상품 목록 |
| GET | /shop/items/{itId} | 상품 + 옵션 |
| GET/POST/PATCH/DELETE | /shop/cart | 장바구니 |
| POST | /shop/orders | 주문서 생성 → `odId` 발급 |
| POST | /shop/orders/{odId}/pay | PG 결제 요청 파라미터 생성 |
| POST | /shop/pg/{pg}/callback | PG 리턴/노티 (서명 검증) |
| GET | /shop/orders/{odId} | 주문 조회 (비회원: odPwd) |
| POST | /shop/orders/{odId}/cancel | 취소 |
| GET/POST | /shop/items/{itId}/reviews · /qa | 후기/문의 |

### 관리자 (`/api/v1/adm`, `g5_auth` 메뉴 코드 검사)
`/adm/config`, `/adm/members`, `/adm/boards`, `/adm/groups`, `/adm/points`, `/adm/shop/items`, `/adm/shop/orders`, `/adm/shop/stats`, `/adm/license`, `/adm/widgets`

### 라이센스
| 메서드 | 경로 | 설명 |
|---|---|---|
| POST | /license/activate | `{licenseKey, domain}` |
| GET | /license/status | 상태 조회 |

## 4. WebSocket
- `GET /ws` (쿼리 `?token=` 또는 쿠키 인증)
- 이벤트: `memo.new`, `comment.new`(내 글에 댓글), `order.new`(관리자), `order.status`
```json
{ "type": "memo.new", "payload": { "meId": 123, "from": "admin" } }
```

## 5. 에러 코드 (초안)
`AUTH_EMPTY`, `AUTH_INVALID`, `AUTH_REQUIRED`, `AUTH_EXPIRED`, `AUTH_REUSED`, `AUTH_TOO_MANY`, `MEMBER_LEFT`, `MEMBER_INTERCEPTED`, `MEMBER_EMAIL_UNCERTIFIED`, `DB_DISABLED`, `BOARD_NOT_FOUND`, `BOARD_NO_PERMISSION`, `POST_SECRET`, `POINT_NOT_ENOUGH`, `WRITE_DELAY`(cf_delay_sec), `BOARD_NOT_FOUND`, `BOARD_GROUP_DENIED`, `BOARD_CERT_REQUIRED`, `BOARD_ADULT_REQUIRED`, `POST_NOT_FOUND`, `POST_PASSWORD_WRONG`, `FILE_NOT_FOUND`, `FILE_TOKEN_INVALID`, `FILE_NO_PERMISSION`, `SHOP_SOLDOUT`, `SHOP_STOCK`, `PG_VERIFY_FAIL`, `LICENSE_INVALID`
