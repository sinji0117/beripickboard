use crate::config::Config;
use anyhow::Result;
use sqlx::mysql::{MySqlConnectOptions, MySqlPool, MySqlPoolOptions};
use std::{str::FromStr, sync::Arc, time::Duration};

#[derive(Clone)]
pub struct AppState(pub Arc<Inner>);

pub struct Inner {
    pub config: Config,
    pub db: Option<MySqlPool>,
    pub redis: Option<redis::aio::ConnectionManager>,
}

impl AppState {
    pub async fn new(config: Config) -> Result<Self> {
        let db = match &config.database_url {
            Some(url) => Some(connect_db(url)?),
            None => {
                tracing::warn!("DATABASE_URL 미설정 — DB 없이 기동");
                None
            }
        };
        let redis = match &config.redis_url {
            Some(url) => match redis::Client::open(url.as_str()) {
                Ok(c) => match redis::aio::ConnectionManager::new(c).await {
                    Ok(m) => Some(m),
                    Err(e) => {
                        tracing::warn!(error = %e, "Redis 연결 실패 — 캐시 없이 기동");
                        None
                    }
                },
                Err(e) => {
                    tracing::warn!(error = %e, "REDIS_URL 형식 오류");
                    None
                }
            },
            None => None,
        };
        Ok(Self(Arc::new(Inner { config, db, redis })))
    }

    #[cfg(test)]
    pub fn without_backends(config: Config) -> Self {
        Self(Arc::new(Inner {
            config,
            db: None,
            redis: None,
        }))
    }
}

/// g5 호환 연결: sql_mode 를 비워 `0000-00-00` 기본값을 허용하고, utf8mb4 로 통신한다.
fn connect_db(url: &str) -> Result<MySqlPool> {
    let opts = MySqlConnectOptions::from_str(url)?
        .charset("utf8mb4")
        .pipes_as_concat(false);
    let pool = MySqlPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(5))
        .after_connect(|conn, _| {
            Box::pin(async move {
                sqlx::query("SET SESSION sql_mode = ''")
                    .execute(&mut *conn)
                    .await?;
                Ok(())
            })
        })
        .connect_lazy_with(opts);
    Ok(pool)
}
