# 10. 카드·간편결제와 수수료·정산

## 1. PG 지원 범위 (영카트 5.4.5.5.1 기준 모듈)
| PG | 영카트 원본 | 우선순위 |
|---|---|---|
| 토스페이먼츠 | shop/toss, settle_toss.inc.php | 1차 |
| KCP | shop/kcp (REST API 포함) | 1차 |
| KG이니시스 | shop/inicis | 2차 |
| 나이스페이 | shop/nicepay | 2차 |
| LG U+(구 토스 이전) | shop/lg | 레거시 데이터 조회만 |
| 카카오페이 / 네이버페이 | shop/kakaopay, shop/naverpay | 2차 |

설정값은 `g5_shop_default` 기존 컬럼을 그대로 읽는다: `de_pg_service`(사용 PG), `de_card_test`(테스트 모드), `de_kcp_mid`·`de_kcp_site_key`·`de_kcp_payment_api`, `de_inicis_mid`·`de_inicis_sign_key`·`de_inicis_iniapi_key`·`de_inicis_iniapi_iv`, `de_nicepay_mid`·`de_nicepay_key`, `de_naverpay_*` 등. > TODO(확인): 토스페이먼츠 키 컬럼명.

## 2. 결제 흐름
```text
장바구니(ct_status=쇼핑) → 주문서 작성 → POST /shop/orders (od_id 발급, 금액 서버 재계산)
 → PG 결제창 → PG 리턴 → 서버 승인 요청(금액 검증) → 트랜잭션:
    g5_shop_order INSERT/UPDATE (od_receipt_price, od_tno, od_app_no, od_settle_case, od_status)
    g5_shop_cart ct_status 쇼핑→주문/입금
    재고 차감(ct_stock_use), 쿠폰 사용(g5_shop_coupon_log), 포인트 사용(g5_point)
    g5_shop_pg_approval_log 기록
 → 실패 시 PG 취소 API 호출 (망취소)
```
- **금액은 항상 서버에서 재계산**(상품가+옵션가−쿠폰−포인트+배송비). 클라이언트 금액 신뢰 금지.
- 가상계좌: 입금 통보(노티) 수신 시 `od_status` 주문→입금. 노티는 PG IP 화이트리스트 + 서명 검증.
- 중복 승인 방지: `od_id` 단위 Redis 락 + DB 유니크.

## 3. 주문 상태 (g5 값 그대로)
`주문` → `입금` → `준비` → `배송` → `완료` / `취소`, `반품`, `품절`
- 부분 취소: 장바구니 행(`ct_status`) 단위 취소 + `od_cancel_price` 누적, PG 부분취소 API
- 미수금: `od_misu = 주문총액 − od_receipt_price − od_receipt_point + od_cancel_price…` (영카트 계산식과 동일하게 구현, 14 문서에서 PHP 결과와 비교 검증)

## 4. 수수료·정산 (beripickboard 신규)
```text
bp_pg_fee(pg, method, rate_bp, fixed_fee, vat_included, valid_from)
  method: card, vbank, bank, easy_kakao, easy_naver, easy_toss, phone
  rate_bp: 수수료율 (basis point, 3.3% = 330)
bp_settlement(id, pg, settle_date, od_id, method, amount, fee, vat, net, status, raw_json)
```
- 일 배치: PG 정산 내역(파일/API) 수집 → `od_tno` 로 주문 매칭 → `bp_settlement` 적재
- 리포트: 기간별 매출 / 수수료 / 순입금, 불일치(주문 있음·정산 없음, 금액 차이) 목록
- 금액은 정수(원). 수수료 반올림 규칙은 PG별 설정(`rounding: floor|round|ceil`)

## 5. 보안 체크
- PG 키는 `.env` 또는 암호화 저장, 관리자 화면 마스킹
- 결제 관련 요청 모두 `tracing` 감사 로그 (카드번호·개인정보 제외)
- 테스트 모드(`od_test`) 주문은 정산·통계에서 제외
