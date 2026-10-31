use super::jwt::{self, Claims};
use crate::{error::AppError, state::AppState};
use axum::{
    extract::{ConnectInfo, FromRequestParts},
    http::{HeaderMap, header, request::Parts},
};
use std::net::SocketAddr;

/// 로그인한 회원 (Access JWT 검증 완료).
pub struct CurrentMember(pub Claims);

impl FromRequestParts<AppState> for CurrentMember {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(|| AppError::unauthorized("AUTH_REQUIRED", "로그인이 필요합니다."))?;
        jwt::verify(&state.0.config.jwt_secret, token)
            .map(CurrentMember)
            .ok_or_else(|| AppError::unauthorized("AUTH_EXPIRED", "로그인이 만료되었습니다."))
    }
}

/// 로그인했으면 Some (토큰이 없거나 만료면 None — 비회원으로 처리)
pub struct OptionalMember(pub Option<Claims>);

impl FromRequestParts<AppState> for OptionalMember {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let claims = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .and_then(|t| jwt::verify(&state.0.config.jwt_secret, t));
        Ok(Self(claims))
    }
}

/// 요청자 IP / User-Agent
pub struct ClientMeta {
    pub ip: String,
    pub user_agent: String,
}

impl FromRequestParts<AppState> for ClientMeta {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let peer = parts
            .extensions
            .get::<ConnectInfo<SocketAddr>>()
            .map(|c| c.0.ip().to_string())
            .unwrap_or_default();
        let ip = if state.0.config.trust_proxy {
            forwarded_ip(&parts.headers).unwrap_or(peer)
        } else {
            peer
        };
        let user_agent = parts
            .headers
            .get(header::USER_AGENT)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_owned();
        Ok(Self { ip, user_agent })
    }
}

fn forwarded_ip(h: &HeaderMap) -> Option<String> {
    h.get("x-forwarded-for")?
        .to_str()
        .ok()?
        .split(',')
        .next()
        .map(|s| s.trim().to_owned())
}

/// Cookie 헤더에서 값 하나 꺼내기
pub fn cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find_map(|v| {
            v.split(';').find_map(|kv| {
                let (k, val) = kv.trim().split_once('=')?;
                (k == name).then(|| val.to_owned())
            })
        })
}
