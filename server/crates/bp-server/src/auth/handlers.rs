use super::{
    REFRESH_COOKIE, REFRESH_COOKIE_PATH, REFRESH_TTL_AUTO_SECS, REFRESH_TTL_SESSION_SECS,
    extract::{ClientMeta, CurrentMember, cookie},
    jwt, refresh,
};
use crate::{
    error::{AppError, AppResult},
    g5::{
        Now,
        config::{self as g5config, G5Config},
        member::{self, Member},
        point::{self, PointInput},
    },
    state::AppState,
};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header::SET_COOKIE},
    response::{AppendHeaders, IntoResponse},
};
use bp_g5compat::{
    member::{email_certified, is_super_admin, ymd_reached},
    password::{self, Verify},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

/// 존재하지 않는 아이디에도 같은 시간이 걸리도록 검증할 더미 해시 (사용자 열거 방지)
const DUMMY_HASH: &str = "sha256:12000:z2WJyEE9ecDTA40/4W8y4l9mKw56lOC6:JPhpyZBWkRauawQzF2MiomGvwB4LFjFV";
const MSG_INVALID: &str =
    "가입된 회원아이디가 아니거나 비밀번호가 틀립니다. 비밀번호는 대소문자를 구분합니다.";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginReq {
    mb_id: String,
    password: String,
    #[serde(default)]
    auto_login: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberView {
    mb_id: String,
    mb_nick: String,
    mb_name: String,
    mb_email: String,
    mb_level: i64,
    mb_point: i64,
    is_admin: Option<&'static str>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenRes {
    access_token: String,
    expires_in: i64,
    member: MemberView,
}

fn db(s: &AppState) -> AppResult<&MySqlPool> {
    s.0.db
        .as_ref()
        .ok_or_else(|| AppError::unavailable("DB_DISABLED", "DB가 설정되지 않았습니다."))
}

fn view(m: &Member, cfg: &G5Config) -> MemberView {
    MemberView {
        mb_id: m.mb_id.clone(),
        mb_nick: m.mb_nick.clone(),
        mb_name: m.mb_name.clone(),
        mb_email: m.mb_email.clone(),
        mb_level: m.mb_level,
        mb_point: m.mb_point,
        is_admin: is_super_admin(&cfg.cf_admin, &m.mb_id).then_some("super"),
    }
}

fn fmt_ymd(ymd: &str) -> String {
    if ymd.len() == 8 {
        format!("{}년 {}월 {}일", &ymd[0..4], &ymd[4..6], &ymd[6..8])
    } else {
        ymd.to_owned()
    }
}

/// login_check.php 의 차단·탈퇴·메일인증 검사
fn check_status(m: &Member, cfg: &G5Config, now: Now) -> AppResult<()> {
    let today = now.ymd_compact();
    if ymd_reached(&m.mb_intercept_date, &today) {
        return Err(AppError::forbidden(
            "MEMBER_INTERCEPTED",
            format!(
                "회원님의 아이디는 접근이 금지되어 있습니다. 처리일 : {}",
                fmt_ymd(&m.mb_intercept_date)
            ),
        ));
    }
    if ymd_reached(&m.mb_leave_date, &today) {
        return Err(AppError::forbidden(
            "MEMBER_LEFT",
            format!(
                "탈퇴한 아이디이므로 접근하실 수 없습니다. 탈퇴일 : {}",
                fmt_ymd(&m.mb_leave_date)
            ),
        ));
    }
    if cfg.cf_use_email_certify != 0 && !email_certified(&m.mb_email_certify) {
        return Err(AppError::forbidden(
            "MEMBER_EMAIL_UNCERTIFIED",
            format!("{} 메일로 메일인증을 받으셔야 로그인 가능합니다.", m.mb_email),
        ));
    }
    Ok(())
}

/// common.php: 오늘 첫 접속이면 로그인 포인트 지급 + 접속일시/IP 갱신
async fn touch_today_login(s: &AppState, cfg: &G5Config, m: &Member, ip: &str, now: Now) -> AppResult<()> {
    let ymd = now.ymd();
    if m.mb_today_login.get(0..10) == Some(ymd.as_str()) {
        return Ok(());
    }
    let (db, p) = (db(s)?, &s.0.config.prefix);
    if cfg.cf_login_point != 0 {
        let pt = i32::try_from(cfg.cf_login_point).unwrap_or(0);
        point::insert_point(
            db,
            p,
            cfg,
            now,
            PointInput {
                mb_id: &m.mb_id,
                point: pt,
                content: &format!("{ymd} 첫로그인"),
                rel_table: "@login",
                rel_id: &m.mb_id,
                rel_action: &ymd,
            },
        )
        .await?;
    }
    member::update_today_login(db, p, &m.mb_id, &now.ymdhis(), ip).await?;
    Ok(())
}

fn refresh_cookie(token: &str, auto_login: bool, secure: bool) -> String {
    let max_age = if auto_login {
        format!("; Max-Age={REFRESH_TTL_AUTO_SECS}")
    } else {
        String::new()
    };
    let secure = if secure { "; Secure" } else { "" };
    format!("{REFRESH_COOKIE}={token}; Path={REFRESH_COOKIE_PATH}; HttpOnly; SameSite=Lax{secure}{max_age}")
}

fn clear_cookie() -> String {
    format!("{REFRESH_COOKIE}=; Path={REFRESH_COOKIE_PATH}; HttpOnly; SameSite=Lax; Max-Age=0")
}

async fn issue_tokens(
    s: &AppState,
    cfg: &G5Config,
    mb_id: &str,
    family_id: &str,
    auto_login: bool,
    meta: &ClientMeta,
) -> AppResult<impl IntoResponse + use<>> {
    let db = db(s)?;
    let now = Utc::now();
    let m = member::find(db, &s.0.config.prefix, mb_id)
        .await?
        .ok_or_else(|| AppError::unauthorized("AUTH_INVALID", MSG_INVALID))?;
    let mv = view(&m, cfg);
    let ttl = if auto_login {
        REFRESH_TTL_AUTO_SECS
    } else {
        REFRESH_TTL_SESSION_SECS
    };
    let rt = refresh::create(
        db,
        refresh::NewToken {
            family_id,
            mb_id: &m.mb_id,
            auto_login,
            user_agent: &meta.user_agent,
            ip: &meta.ip,
            ttl_secs: ttl,
        },
        now,
    )
    .await?;
    let access = jwt::issue(
        &s.0.config.jwt_secret,
        &m.mb_id,
        m.mb_level,
        mv.is_admin,
        now.timestamp(),
    )?;
    let set_cookie = refresh_cookie(&rt, auto_login, s.0.config.is_production());
    Ok((
        AppendHeaders([(SET_COOKIE, set_cookie)]),
        Json(
            serde_json::json!({ "data": TokenRes { access_token: access, expires_in: super::ACCESS_TTL_SECS, member: mv } }),
        ),
    ))
}

/// POST /api/v1/auth/login
pub async fn login(
    State(s): State<AppState>,
    meta: ClientMeta,
    Json(req): Json<LoginReq>,
) -> AppResult<impl IntoResponse> {
    // login_check.php 와 동일하게 앞뒤 공백 제거
    let (mb_id, pw) = (req.mb_id.trim(), req.password.trim());
    if mb_id.is_empty() || pw.is_empty() {
        return Err(AppError::bad_request(
            "AUTH_EMPTY",
            "회원아이디나 비밀번호가 공백이면 안됩니다.",
        ));
    }
    if s.0.limiter.is_blocked(&meta.ip, mb_id) {
        return Err(AppError::too_many(
            "AUTH_TOO_MANY",
            "로그인 시도가 너무 많습니다. 잠시 후 다시 시도하세요.",
        ));
    }
    let (db, p) = (db(&s)?, &s.0.config.prefix);
    let cfg = g5config::load(db, p).await?;
    let found = member::find(db, p, mb_id).await?;

    let verified = match &found {
        Some(m) => password::verify(pw, &m.mb_password),
        None => {
            let _ = password::verify(pw, DUMMY_HASH);
            Verify::Mismatch
        }
    };
    let m = match (found, &verified) {
        (Some(m), Verify::Ok | Verify::OkNeedsUpgrade) => m,
        (Some(m), Verify::Unsupported) => {
            tracing::warn!(mb_id = %m.mb_id, "검증할 수 없는 비밀번호 해시 형식 (OLD_PASSWORD 등) — 재설정 필요");
            s.0.limiter.fail(&meta.ip, mb_id);
            return Err(AppError::unauthorized("AUTH_INVALID", MSG_INVALID));
        }
        _ => {
            s.0.limiter.fail(&meta.ip, mb_id);
            return Err(AppError::unauthorized("AUTH_INVALID", MSG_INVALID));
        }
    };
    let now = Now::new(s.0.config.tz);
    check_status(&m, &cfg, now)?;
    s.0.limiter.success(&meta.ip, mb_id);

    if verified == Verify::OkNeedsUpgrade && s.0.config.rehash_on_login {
        let mut salt = [0u8; password::PBKDF2_SALT_BYTES];
        getrandom::fill(&mut salt).map_err(|e| anyhow::anyhow!("난수 생성 실패: {e}"))?;
        member::update_password(db, p, &m.mb_id, &password::create_hash(pw, &salt)).await?;
        tracing::info!(mb_id = %m.mb_id, "비밀번호 해시를 g5 sha256 형식으로 승급");
    }
    touch_today_login(&s, &cfg, &m, &meta.ip, now).await?;
    let _ = refresh::purge_expired(db, Utc::now()).await;

    // g5: 최고관리자는 자동로그인 금지
    let auto_login = req.auto_login && !is_super_admin(&cfg.cf_admin, &m.mb_id);
    let family = refresh::random_hex(16)?;
    issue_tokens(&s, &cfg, &m.mb_id, &family, auto_login, &meta).await
}

fn auth_fail(code: &'static str, msg: &str) -> axum::response::Response {
    (
        AppendHeaders([(SET_COOKIE, clear_cookie())]),
        AppError::unauthorized(code, msg),
    )
        .into_response()
}

/// POST /api/v1/auth/refresh — 리프레시 토큰 회전
pub async fn refresh_token(
    State(s): State<AppState>,
    meta: ClientMeta,
    headers: HeaderMap,
) -> AppResult<axum::response::Response> {
    let Some(token) = cookie(&headers, REFRESH_COOKIE).filter(|t| !t.is_empty()) else {
        return Ok(auth_fail("AUTH_REQUIRED", "로그인이 필요합니다."));
    };
    let (db, p) = (db(&s)?, &s.0.config.prefix);
    let now_utc = Utc::now();
    let Some(st) = refresh::find(db, &token).await? else {
        return Ok(auth_fail("AUTH_INVALID", "로그인 정보가 올바르지 않습니다."));
    };
    if st.revoked_at.is_some() {
        // 이미 사용된 토큰이 다시 옴 → 탈취 의심, 패밀리 전체 폐기
        refresh::revoke_family(db, &st.family_id, now_utc).await?;
        tracing::warn!(mb_id = %st.mb_id, "리프레시 토큰 재사용 감지 — 세션 폐기");
        return Ok(auth_fail(
            "AUTH_REUSED",
            "다른 곳에서 로그인 정보가 사용되어 로그아웃되었습니다.",
        ));
    }
    if st.expires_at < now_utc.naive_utc() {
        return Ok(auth_fail("AUTH_EXPIRED", "로그인이 만료되었습니다."));
    }
    if !refresh::revoke_one(db, &token, now_utc).await? {
        refresh::revoke_family(db, &st.family_id, now_utc).await?;
        return Ok(auth_fail(
            "AUTH_REUSED",
            "다른 곳에서 로그인 정보가 사용되어 로그아웃되었습니다.",
        ));
    }
    let cfg = g5config::load(db, p).await?;
    let Some(m) = member::find(db, p, &st.mb_id).await? else {
        refresh::revoke_family(db, &st.family_id, now_utc).await?;
        return Ok(auth_fail("AUTH_INVALID", MSG_INVALID));
    };
    let now = Now::new(s.0.config.tz);
    if let Err(e) = check_status(&m, &cfg, now) {
        refresh::revoke_family(db, &st.family_id, now_utc).await?;
        return Ok((AppendHeaders([(SET_COOKIE, clear_cookie())]), e).into_response());
    }
    touch_today_login(&s, &cfg, &m, &meta.ip, now).await?;
    Ok(
        issue_tokens(&s, &cfg, &m.mb_id, &st.family_id, st.auto_login != 0, &meta)
            .await?
            .into_response(),
    )
}

/// POST /api/v1/auth/logout
pub async fn logout(State(s): State<AppState>, headers: HeaderMap) -> AppResult<impl IntoResponse> {
    if let (Some(token), Ok(db)) = (cookie(&headers, REFRESH_COOKIE), db(&s))
        && let Some(st) = refresh::find(db, &token).await?
    {
        refresh::revoke_family(db, &st.family_id, Utc::now()).await?;
    }
    Ok((
        StatusCode::NO_CONTENT,
        AppendHeaders([(SET_COOKIE, clear_cookie())]),
    ))
}

/// GET /api/v1/members/me
pub async fn me(State(s): State<AppState>, CurrentMember(c): CurrentMember) -> AppResult<impl IntoResponse> {
    let (db, p) = (db(&s)?, &s.0.config.prefix);
    let cfg = g5config::load(db, p).await?;
    let m = member::find(db, p, &c.sub)
        .await?
        .ok_or_else(|| AppError::unauthorized("AUTH_INVALID", "회원 정보를 찾을 수 없습니다."))?;
    Ok(Json(serde_json::json!({ "data": view(&m, &cfg) })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cookie_flags() {
        let c = refresh_cookie("abc", true, true);
        assert!(c.contains("HttpOnly") && c.contains("Secure") && c.contains("Max-Age=1209600"));
        assert!(!refresh_cookie("abc", false, false).contains("Max-Age"));
        assert_eq!(fmt_ymd("20261031"), "2026년 10월 31일");
    }
}
