# 14. 테스트

## 1. 계층
| 종류 | 도구 | 대상 |
|---|---|---|
| 단위 | `cargo test` | 해시, wr_reply 계산, 금액 계산, 권한 판정 |
| 통합 | `cargo test --features it` + testcontainers(MariaDB/Redis) | API ↔ DB |
| 프론트 | Vitest + Testing Library | 컴포넌트 |
| E2E | Playwright | 가입→글쓰기→주문→결제(테스트 PG) |
| **호환(골든)** | PHP g5 컨테이너 vs bp-server | 같은 입력 → 같은 DB 결과 |

## 2. g5 호환 테스트 (핵심)
같은 DB 스냅샷 두 벌을 만들고, 동일 시나리오를 **PHP 그누보드**와 **beripickboard**에 각각 실행한 뒤 DB 행을 비교한다.

| 시나리오 | 비교 대상 |
|---|---|
| 글쓰기 / 답글 3단계 / 댓글·대댓글 | g5_write_* 의 wr_num, wr_reply, wr_parent, wr_comment, wr_comment_reply, g5_board.bo_count_* |
| 글 삭제(답글·댓글 있는 글) | 남는 행, 카운터, g5_board_new |
| 포인트 (쓰기·읽기·다운로드·소멸) | g5_point 행, mb_point |
| 회원가입 / 탈퇴 | g5_member 필드 |
| 장바구니 → 주문 → 부분취소 | g5_shop_cart.ct_status, g5_shop_order 금액 필드(od_misu 포함) |
| 쿠폰 + 포인트 결제 | g5_shop_coupon_log, g5_point |

비교 제외 컬럼: `*_datetime`, `*_ip`, `wr_last`, 자동 증가 id(대응표로 매핑).

### 2.1 실행 도구
`tools/compat/` — PHP 그누보드를 같은 DB 로 띄우고 PHP 화면으로 데이터를 만든 뒤 비교 (README 참고).

### 2.2 v0.0.4 결과
| 항목 | 결과 |
|---|---|
| 목록 7가지(1~3쪽, 제목/내용 검색, 조회수·날짜 정렬) wr_id 순서 | 동일 |
| 글 번호(공지 포함, 2쪽 공지 보정) | 동일 |
| 읽기·다운로드 포인트, FIFO 사용포인트, 소멸 포인트 `g5_point` 행 | 동일 (소멸 uniqid 값 제외) |
| 비밀글(회원 글은 작성자 비밀번호), 비밀댓글 | 동일 규칙 |

## 3. 비밀번호 호환 단위 테스트
- g5 PHP `create_hash('test1234')` 로 만든 실제 해시 샘플 N개 → Rust 검증 통과
- MySQL `SELECT PASSWORD('test1234')` 결과 → 통과
- 잘못된 비밀번호 → 실패, 상수 시간 비교 확인

## 4. 마이그레이션 데이터 검증 (`bp-migrate verify`)
- 테이블별 행 수 (원본 덤프 vs 전환 후)
- 테이블별 체크섬 (`CHECKSUM TABLE` 또는 PK 정렬 후 해시)
- 참조 무결성: `g5_write_*.mb_id` ∈ g5_member(빈 값=비회원 허용), `g5_board_file` ↔ 실제 파일 존재
- 0일자·문자셋 변환 후 깨짐 검사 (변환 전후 `HEX()` 비교 표본)
- 이윰보드 위젯 변환: 원본 EB 항목 수 = bp_widget 수

## 5. CI
PR마다: fmt → clippy → 단위 → 통합 → 프론트 → 호환(주요 시나리오). 태그 배포 전: 전체 E2E + 호환 전체.
