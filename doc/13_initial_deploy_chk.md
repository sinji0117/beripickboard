# 13. 첫 배포 체크리스트

## 1. 배포 전
- [ ] 운영 DB 전체 백업 (`mysqldump --single-transaction`) + `data/` 백업, 복원 리허설 1회
- [ ] `bp-migrate check` 결과: 누락 컬럼 0, 구버전 마이그레이션 적용 완료
- [ ] MyISAM → InnoDB 변환 완료 여부 (`SELECT TABLE_NAME, ENGINE FROM information_schema.TABLES WHERE TABLE_SCHEMA=DATABASE()`)
- [ ] 문자셋 utf8mb4 변환 여부 결정·적용
- [ ] 14 문서의 마이그레이션 검증 테스트 전 항목 통과

## 2. 환경변수 · 보안
- [ ] `.env` 권한 600, 소유자 `beripick`
- [ ] `JWT_SECRET` 32바이트 이상 무작위, 스테이징과 다른 값
- [ ] `APP_ENV=production`, 디버그 로그 OFF
- [ ] DB 계정은 해당 DB 권한만 (GRANT ALL ON *.* 금지)
- [ ] PG 키가 테스트 키가 아닌지 / `de_card_test` 값 확인
- [ ] 라이센스 키 등록·도메인 인증 완료
- [ ] CORS_ORIGIN 운영 도메인만

## 3. 네트워크
- [ ] 외부 개방 포트: 80, 443만. 8080(앱)·3306(DB)·6379(Redis) 외부 차단
- [ ] Redis `bind 127.0.0.1` + 비밀번호
- [ ] PG 노티 수신 URL 접근 가능 + PG사 IP 화이트리스트
- [ ] SSL 인증서 정상, HTTP→HTTPS 리다이렉트

## 4. 데이터 정합성 (전환 직후)
- [ ] 회원 수, 게시판별 글/댓글 수가 `bo_count_write`/`bo_count_comment`와 일치
- [ ] 샘플 회원 10명 로그인 성공 (PBKDF2 / 구 PASSWORD() 계정 각각)
- [ ] 첨부파일 다운로드, 에디터 이미지, 상품 이미지 표시
- [ ] 테스트 주문 결제 → 취소까지 1회
- [ ] `mb_point` = `SUM(g5_point.po_point)` 표본 검사

## 5. URL · SEO
- [ ] `/bbs/board.php?bo_table=…&wr_id=…`, `/shop/item.php?it_id=…` 등 구 URL 301 리다이렉트
- [ ] `cf_bbs_rewrite`(짧은 주소) 사용 사이트는 해당 패턴도 리다이렉트
- [ ] sitemap.xml, robots.txt

## 6. 모니터링
- [ ] 헬스체크 감시, 에러 로그 수집, 디스크(data/) 사용량 알림
- [ ] 일일 DB 백업 스케줄
