# 05. 플랫폼 디자인 설계서

## 1. 개념 매핑 (g5/EB4 → beripickboard)
| g5 / 이윰보드 | beripickboard | 비고 |
|---|---|---|
| `theme/{name}/` | `web/src/themes/{name}/` | `g5_config.cf_theme` 로 선택 |
| `head.php` / `tail.php` | `layout/Head.tsx`, `layout/Tail.tsx` | 레이아웃 컴포넌트 |
| `skin/board/{skin}/list.skin.php` 등 | `skin/board/{skin}/List.tsx`, `View.tsx`, `Write.tsx` | 파일 단위 1:1 |
| `skin/latest/{skin}` | `skin/latest/{skin}` | 최신글 = 위젯 |
| `mobile/skin/…` | 반응형 단일 컴포넌트 | `bo_mobile_skin`은 별도 지정 시만 사용 |
| 이윰 EB 슬라이더 / EB 콘텐츠 / EB 최신글 | `widgets/` (Slider, Contents, Latest) | `bp_widget` 저장 |
| 이윰 메뉴 관리 | `g5_menu` + 이윰 메뉴 테이블 매핑 | > TODO(확인) |
| 이윰 레이아웃 설정(사이드 위치 등) | 테마 `theme.config.ts` + 관리자 설정 | |

## 2. 테마 구조
```text
themes/basic/
├── theme.config.ts     # 이름, 버전, 지원 슬롯, 기본 스킨
├── tokens.css          # Tailwind @theme 토큰
├── layout/             # Head, Tail, Sidebar, Footer
├── pages/              # Index(메인), Content(내용관리) 등
└── skin/
    ├── board/basic|gallery|webzine
    ├── latest/ member/ outlogin/ poll/ popular/ search/ visit/ connect/ faq/ qa/ new/ content/
    └── shop/           # 영카트 스킨 (list, item, cart, orderform …)
```
skin 하위 폴더명은 g5 `skin/` 폴더명(board, connect, content, faq, latest, member, new, outlogin, poll, popular, qa, search, shop, social, visit)과 동일하게 둔다.

## 3. 위젯 시스템
- 슬롯: 테마가 `theme.config.ts`에 `slots: ['main_top', 'main_left', 'sidebar', 'footer']` 처럼 선언
- 위젯 타입: `latest`(게시판 최신글), `slider`, `banner`, `html`, `shop_items`(상품 진열 `it_type1~5`), `poll`, `popular`, `visit`, `outlogin`
- 저장: `bp_widget(id, type, title, options_json, theme, created_at)`, `bp_widget_slot(slot, widget_id, sort, device)`
- 관리자에서 드래그 앤 드롭 배치, 미리보기 실시간 반영
- 이윰 EB 콘텐츠/슬라이더/최신글 데이터는 이전 시 `bp_widget`으로 변환 (→ 06, 14)

## 4. 관리자 UI
- g5 관리자 메뉴 코드와 동일한 트리 (100/200/300/400/500/900), 권한은 `g5_auth`
- 좌측 메뉴 + 상단 검색 + 대시보드(오늘 가입/글/주문/매출)
- 테이블 목록 공통 컴포넌트: 검색, 정렬, 페이지, 일괄 처리(선택 수정/삭제) — g5 관리자 사용 습관 유지

## 5. 라이센스 인증 UI/UX
- 최초 설치 마법사: DB 연결 → **기존 g5 DB 감지(g5_config 존재 여부)** → 라이센스 키 입력 → 도메인 확인 → 관리자 계정 확인
- 관리자 > 환경설정 > 라이센스: 키, 등록 도메인, 만료일, 상태(정품/만료/미인증), 재인증 버튼
- 미인증 시: 사이트는 정상 동작, 관리자 상단 배너 + 하단 "Powered by beripickboard (미인증)" 표기. **사이트 차단은 하지 않음**(데이터 손실·장애 방지)
- 검증 방식: Ed25519 서명된 라이선스 파일(오프라인 검증) + 주기적 온라인 확인(실패해도 유예 기간 14일)
- 허용 도메인: 정식 도메인 1개 + `localhost`, `127.0.0.1`, `*.test` 개발용 자동 허용
