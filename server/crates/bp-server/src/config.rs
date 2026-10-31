use anyhow::{Context, Result, bail};
use bp_g5compat::table::Prefix;
use chrono::FixedOffset;
use std::{env, net::SocketAddr};

#[derive(Debug, Clone)]
pub struct Config {
    pub app_env: String,
    pub addr: SocketAddr,
    pub database_url: Option<String>,
    pub redis_url: Option<String>,
    pub prefix: Prefix,
    pub data_path: String,
    pub cors_origin: Option<String>,
    /// JWT 서명 키 (32바이트 이상)
    pub jwt_secret: Vec<u8>,
    /// g5 서버 시간대. 그누보드 기본 `Asia/Seoul` (+09:00)
    pub tz: FixedOffset,
    /// 로그인 시 약한 해시(PASSWORD(), sha1)를 g5 sha256 형식으로 재저장할지.
    /// 공존 기간에 PHP 쪽 G5_STRING_ENCRYPT_FUNCTION 이 create_hash 일 때만 켠다.
    pub rehash_on_login: bool,
    /// 리버스 프록시 뒤에서 X-Forwarded-For 를 신뢰할지
    pub trust_proxy: bool,
}

impl Config {
    /// `.env` 및 환경변수에서 읽는다 (doc/07_local_development.md).
    pub fn from_env() -> Result<Self> {
        let get = |k: &str| env::var(k).ok().filter(|v| !v.trim().is_empty());
        let flag = |k: &str| get(k).is_some_and(|v| matches!(v.as_str(), "1" | "true" | "yes"));
        let app_env = get("APP_ENV").unwrap_or_else(|| "local".into());
        let addr = get("APP_ADDR").unwrap_or_else(|| "127.0.0.1:8080".into());

        let jwt_secret = match get("JWT_SECRET") {
            Some(s) if s.len() >= 32 && s != "change-me-32bytes-min" => s.into_bytes(),
            Some(_) | None if app_env == "production" => {
                bail!("운영 환경에서는 32바이트 이상의 JWT_SECRET 이 필요합니다")
            }
            _ => {
                tracing::warn!("JWT_SECRET 미설정/약함 — 개발용 임시 키 사용 (재시작 시 로그인 초기화)");
                let mut k = vec![0u8; 32];
                getrandom::fill(&mut k).map_err(|e| anyhow::anyhow!("난수 생성 실패: {e}"))?;
                k
            }
        };
        let tz_hours: i32 = get("APP_TZ_OFFSET_HOURS")
            .and_then(|v| v.parse().ok())
            .unwrap_or(9);

        Ok(Self {
            addr: addr
                .parse()
                .with_context(|| format!("APP_ADDR 형식 오류: {addr}"))?,
            database_url: get("DATABASE_URL"),
            redis_url: get("REDIS_URL"),
            prefix: Prefix::new(&get("G5_TABLE_PREFIX").unwrap_or_else(|| "g5_".into()))?,
            data_path: get("DATA_PATH").unwrap_or_else(|| "./.local/data".into()),
            cors_origin: get("CORS_ORIGIN"),
            jwt_secret,
            tz: FixedOffset::east_opt(tz_hours * 3600).context("APP_TZ_OFFSET_HOURS 범위 오류")?,
            rehash_on_login: flag("G5_REHASH_ON_LOGIN"),
            trust_proxy: flag("TRUST_PROXY"),
            app_env,
        })
    }

    pub fn is_production(&self) -> bool {
        self.app_env == "production"
    }

    #[cfg(test)]
    pub fn for_test() -> Self {
        Self {
            app_env: "test".into(),
            addr: ([127, 0, 0, 1], 0).into(),
            database_url: None,
            redis_url: None,
            prefix: Prefix::default(),
            data_path: ".".into(),
            cors_origin: None,
            jwt_secret: b"test-secret-test-secret-test-secret!".to_vec(),
            tz: FixedOffset::east_opt(9 * 3600).unwrap_or_else(|| unreachable!()),
            rehash_on_login: false,
            trust_proxy: false,
        }
    }
}
