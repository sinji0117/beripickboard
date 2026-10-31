# 12. Rpanel로 배포하기

> TODO(확인): Rpanel 대시보드의 배포 기능(업로드/Git 연동/훅) 지원 범위 확인 후 확정.

## 1. 빌드
```bash
# 백엔드 (서버 OS에 맞춘 정적 링크 권장)
cd server && cargo build --release -p bp-server -p bp-migrate --target x86_64-unknown-linux-musl
# 프론트
cd web && pnpm install --frozen-lockfile && pnpm build
```

## 2. 패키징
```text
beripickboard-v0.0.x.tar.gz
├── bp-server
├── bp-migrate
├── web/dist/
├── VERSION
└── checksums.sha256
```
GitHub Actions에서 태그(`v*`) 푸시 시 자동 빌드 → Release 첨부.

## 3. 배포 절차 (무중단에 가깝게)
1. 패키지 업로드 → `releases/{timestamp}/` 에 압축 해제
2. `ln -s ../../shared/.env releases/{ts}/.env`
3. **DB 백업** → `bp-migrate up` (bp_* 테이블만 변경, g5 테이블은 변경 없음)
4. `current` 링크 교체 → Rpanel에서 앱 재시작
5. 헬스체크 `GET /api/v1/health` 200 확인, 실패 시 이전 릴리스로 링크 복구 후 재시작
6. 릴리스 5개 초과분 정리

## 4. 롤백
- 링크를 이전 `releases/` 로 되돌리고 재시작
- `bp_*` 스키마가 바뀐 경우 `bp-migrate down --to <버전>` (down 스크립트 필수 작성)
