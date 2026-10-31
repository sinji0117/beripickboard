# 06. 기능 안내 (이전 대상 기능 명세)

우선순위: **P0** = v0.1.0 필수, **P1** = v0.1.x, **P2** = 이후

## 1. 그누보드5
| 기능 | 원본 위치 | 사용 테이블 | 우선 |
|---|---|---|---|
| 회원가입/로그인/로그아웃/자동로그인 | bbs/register*, login* | g5_member, g5_member_auto_login, g5_login | P0 |
| 정보수정/탈퇴/비밀번호 찾기 | bbs/member_* , password_lost* | g5_member (`mb_leave_date`, `mb_lost_certify`) | P0 |
| 소셜 로그인 | plugin/social | g5_member_social_profiles | P1 |
| 본인인증 | plugin/inicert, kcpcert 등 | g5_cert_history, g5_member_cert_history | P2 |
| 게시판 목록/보기/쓰기/수정/삭제 | bbs/board.php, write*.php | g5_board, g5_write_*, g5_board_new | P0 |
| 답글, 댓글, 대댓글 | write_update, write_comment_update | g5_write_* | P0 |
| 첨부파일, 다운로드 포인트 | bbs/download.php | g5_board_file | P0 |
| 비밀글, 분류, 공지(`bo_notice`) | | g5_board | P0 |
| 추천/비추천 | bbs/good.php | g5_board_good | P0 |
| 검색(전체/게시판) | bbs/search.php | g5_write_*, g5_popular | P0 |
| 새글 | bbs/new.php | g5_board_new | P0 |
| 포인트 지급/차감/소멸 | lib/common.lib.php `insert_point` | g5_point | P0 |
| 쪽지 | bbs/memo* | g5_memo | P1 |
| 스크랩 | bbs/scrap* | g5_scrap | P1 |
| 투표 | bbs/poll* | g5_poll, g5_poll_etc | P1 |
| 내용관리(회사소개 등) | bbs/content.php | g5_content | P0 |
| FAQ / 1:1문의 | bbs/faq, qa* | g5_faq*, g5_qa_* | P1 |
| 팝업 | adm/newwin | g5_new_win | P1 |
| 메뉴 | adm/menu_list | g5_menu | P0 |
| 방문자 통계 | lib/visit | g5_visit, g5_visit_sum | P1 |
| 메일 발송 | adm/mail | g5_mail | P2 |
| 그룹 접근 권한 | | g5_group, g5_group_member | P0 |
| 관리자 권한 | adm/auth_list | g5_auth | P0 |
| 캡차 / 도배 방지 | plugin/kcaptcha | g5_abuse_rate, `cf_delay_sec` | P0 |

## 2. 영카트5
| 기능 | 사용 테이블 | 우선 |
|---|---|---|
| 분류(최대 5단계, `ca_id` 2자리씩) | g5_shop_category | P0 |
| 상품/옵션/추가옵션 (`io_type` 0=선택, 1=추가) | g5_shop_item, g5_shop_item_option | P0 |
| 상품 진열 유형 `it_type1~5` (히트/추천/최신/인기/할인) | g5_shop_item | P0 |
| 장바구니 (`ct_status='쇼핑'`) / 바로구매(`ct_direct`) | g5_shop_cart | P0 |
| 주문서 / 배송지 | g5_shop_order, g5_shop_order_address, g5_shop_order_data | P0 |
| 결제 (→ 10 문서) | PG 로그 테이블 | P0 |
| 주문 상태: 주문→입금→준비→배송→완료, 취소/반품/품절 | g5_shop_order.od_status, g5_shop_cart.ct_status | P0 |
| 배송비 (`it_sc_type`, 지역별 추가 배송비) | g5_shop_sendcost, g5_shop_default | P0 |
| 쿠폰 / 쿠폰존 | g5_shop_coupon*, | P1 |
| 포인트 결제 / 적립 | g5_point, `od_receipt_point` | P0 |
| 사용후기 / 상품문의 | g5_shop_item_use, g5_shop_item_qa | P1 |
| 위시리스트 / 재입고 알림 | g5_shop_wish, g5_shop_item_stocksms | P1 |
| 이벤트 / 배너 | g5_shop_event*, g5_shop_banner | P1 |
| 개인결제 | g5_shop_personalpay | P1 |
| 비회원 주문 조회 (`od_pwd`) | g5_shop_order, g5_shop_order_access | P0 |
| 관리자 주문 관리 / 매출 현황 | 위 전체 | P0 |

## 3. 이윰보드4 (EB4)
> TODO(확인): 아래 항목은 EB4 매뉴얼 기준 기능 범주이며, 테이블·설정 파일 이름은 실제 설치본에서 확정한다.

| 기능 | 이전 방식 | 우선 |
|---|---|---|
| 테마 / 레이아웃 설정 | React 테마로 재구현, 설정값만 이전 | P0 |
| EB 슬라이더 | `bp_widget(type=slider)`로 변환 | P1 |
| EB 콘텐츠 | `bp_widget(type=html/contents)`로 변환 | P1 |
| EB 최신글 | `bp_widget(type=latest)`로 변환 | P0 |
| 이윰 메뉴 | 테이블 그대로 읽기 | P0 |
| 회원 레벨/경험치·활동 내역 | 테이블 그대로 사용 | P1 |
| 태그 | 테이블 그대로 사용 | P2 |
| 게시판 확장 설정(이윰 게시판 옵션) | 테이블 그대로 사용 | P1 |

## 4. beripickboard 독자 기능
| 기능 | 테이블 | 우선 |
|---|---|---|
| 라이센스 등록 / 도메인 정품 인증 | bp_license | P0 |
| 위젯 편집기 (드래그 배치) | bp_widget, bp_widget_slot | P1 |
| 마이그레이션 마법사 (DB 감지 → 검사 → 변환) | bp_schema_version | P0 |
| 정산 리포트 | bp_settlement, bp_pg_fee | P1 |
| 실시간 알림 (WebSocket: 댓글·쪽지·주문) | — | P2 |
