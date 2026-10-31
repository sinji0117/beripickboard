use anyhow::{Context, Result};
use bp_g5compat::table::Prefix;
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
}

impl Config {
    /// `.env` 및 환경변수에서 읽는다 (doc/07_local_development.md).
    pub fn from_env() -> Result<Self> {
        let get = |k: &str| env::var(k).ok().filter(|v| !v.trim().is_empty());
        let addr = get("APP_ADDR").unwrap_or_else(|| "127.0.0.1:8080".into());
        Ok(Self {
            app_env: get("APP_ENV").unwrap_or_else(|| "local".into()),
            addr: addr
                .parse()
                .with_context(|| format!("APP_ADDR 형식 오류: {addr}"))?,
            database_url: get("DATABASE_URL"),
            redis_url: get("REDIS_URL"),
            prefix: Prefix::new(&get("G5_TABLE_PREFIX").unwrap_or_else(|| "g5_".into()))?,
            data_path: get("DATA_PATH").unwrap_or_else(|| "./.local/data".into()),
            cors_origin: get("CORS_ORIGIN"),
        })
    }
}
