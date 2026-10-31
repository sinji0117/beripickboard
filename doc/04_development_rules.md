# 04. 개발 규칙

## 1. 공통
- 언어: 코드·식별자 영어, 주석·문서·커밋 본문 한국어 허용
- g5 컬럼명은 **원본 그대로** 쓴다 (`wr_subject` → Rust 필드도 `wr_subject`). 변환 이름을 만들지 않는다. API 응답에서만 camelCase로 직렬화(`#[serde(rename_all = "camelCase")]`).
- g5 테이블 구조를 바꾸는 PR은 금지. 필요하면 `bp_*` 테이블로.

## 2. Rust
- `cargo fmt` 필수, `cargo clippy --all-targets -- -D warnings` 통과 필수
- `unwrap()`/`expect()` 금지(테스트·초기화 제외). 에러는 `thiserror`로 도메인 에러 정의, 핸들러에서 `AppError → IntoResponse`
- 동적 SQL은 `bp-g5compat::table::write_table(bo_table)` 헬퍼만 사용 (직접 `format!` 금지)
- 금액은 `i64`(원 단위). 부동소수점 금지
- 트랜잭션: 포인트·주문·쿠폰·재고는 반드시 `sqlx::Transaction`
- 로그: `tracing`, 개인정보(mb_password, mb_hp, 주소) 로그 금지

## 3. React 19 / TypeScript
- 함수형 컴포넌트 + hooks만. 클래스 컴포넌트 금지
- `strict: true`, `any` 금지
- 서버 데이터는 TanStack Query, 폼은 React 19 Actions(`useActionState`) 우선
- 사용자 HTML(`wr_content`)은 반드시 `DOMPurify` 정제 후 렌더 (`html1/html2` 옵션 존중)
- 파일·폴더: 컴포넌트 `PascalCase.tsx`, 훅 `useXxx.ts`

## 4. Tailwind CSS 4
- `@theme` 에 디자인 토큰(색·간격·폰트) 정의, 테마별 토큰 파일 분리
- 임의값(`w-[317px]`) 최소화, 반복 패턴은 컴포넌트로 추출
- 다크모드: `data-theme` 속성 기반

## 5. Git
- 브랜치: `main`(배포) ← `develop` ← `feature/{영역}-{요약}`, `fix/…`, `migrate/…`
- 커밋: Conventional Commits — `feat(board): 답글 wr_reply 계산 구현`
- 태그: `v0.0.1` 형식, 주요 기능 추가/변경마다 0.0.1 증가 (→ 08 로드맵)
- PR 체크: fmt, clippy, `cargo test`, `pnpm lint`, `pnpm test`, 호환 테스트(14 문서)
