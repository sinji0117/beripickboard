# 02. 문서 목차

| # | 문서 | 내용 | 상태 |
|---|---|---|---|
| 01 | [설명](01_description.md) | 비전, 핵심 원칙, 범위, 기준 버전 | 초안 v0.0.1 |
| 02 | [문서 목차](02_table_of_contents.md) | 이 문서 | 초안 |
| 03 | [구조](03_architecture.md) | 백엔드/프론트/DB, g5 호환 계층, 공존 운영 | 초안 |
| 04 | [개발 규칙](04_development_rules.md) | Rust/React/Tailwind 규칙, Git 브랜치, 커밋 | 초안 |
| 05 | [플랫폼 디자인 설계서](05_platform_design.md) | 테마·스킨·위젯, 관리자, 라이센스 UI | 초안 |
| 06 | [기능 안내](06_features.md) | 그누보드·영카트·이윰보드 기능별 이전 명세 | 초안 |
| 07 | [로컬 개발 환경](07_local_development.md) | 설치, .env, 기존 DB 붙이기 | 초안 |
| 08 | [로드맵](08_roadmap.md) | v0.0.1 ~ v0.1.0 마일스톤 | 초안 |
| 09 | [API 명세](09_api.md) | REST/WebSocket, 인증 토큰, 에러 규격 | 초안 |
| 10 | [결제·정산](10_payment_settlement.md) | PG 연동, 주문 상태, 수수료·정산 | 초안 |
| 11 | [Rpanel 새 앱 설정](11_rpanel_new_app.md) | 앱 생성, 프록시, SSL | 초안 (Rpanel 화면 확인 필요) |
| 12 | [Rpanel 배포](12_rpanel_deploy.md) | 빌드, 패키징, 무중단 배포 | 초안 (Rpanel 화면 확인 필요) |
| 13 | [첫 배포 체크리스트](13_initial_deploy_chk.md) | 보안, DB 정합성, 방화벽 | 초안 |
| 14 | [테스트](14_testing.md) | 단위/통합/마이그레이션 검증 | 초안 |

## 표기 규칙
- `> TODO(확인)` : 실제 설치본·외부 화면을 보고 확정해야 하는 항목
- 테이블명은 기본 접두사 `g5_` 기준. 실제 사이트의 `G5_TABLE_PREFIX`가 다르면 설정값으로 치환된다.
