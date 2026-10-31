//! 인증: g5 회원 비밀번호로 로그인 → Access JWT + Refresh 토큰 (doc/09_api.md §2).

pub mod extract;
pub mod handlers;
pub mod jwt;
pub mod ratelimit;
pub mod refresh;
pub mod sign;

pub const REFRESH_COOKIE: &str = "bp_rt";
pub const REFRESH_COOKIE_PATH: &str = "/api/v1/auth";
/// Access 토큰 수명
pub const ACCESS_TTL_SECS: i64 = 15 * 60;
/// 자동로그인 리프레시 수명 (doc: 14일)
pub const REFRESH_TTL_AUTO_SECS: i64 = 14 * 24 * 3600;
/// 일반 로그인 리프레시 수명 (브라우저 세션 쿠키 + 서버 측 1일)
pub const REFRESH_TTL_SESSION_SECS: i64 = 24 * 3600;
