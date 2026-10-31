use crate::{error::AppError, state::AppState};
use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;

pub fn router(state: AppState) -> Router {
    let api = Router::new().route("/health", get(health));
    Router::new()
        .nest("/api/v1", api)
        .fallback(|| async { AppError::not_found("NOT_FOUND", "요청한 경로가 없습니다.") })
        .with_state(state)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Health {
    status: &'static str,
    version: &'static str,
    env: String,
    db: &'static str,
    redis: &'static str,
    table_prefix: String,
}

async fn health(State(s): State<AppState>) -> Json<Health> {
    let db = match &s.0.db {
        None => "disabled",
        Some(pool) => match sqlx::query("SELECT 1").execute(pool).await {
            Ok(_) => "ok",
            Err(_) => "error",
        },
    };
    let redis = match s.0.redis.clone() {
        None => "disabled",
        Some(mut c) => match redis::cmd("PING").query_async::<String>(&mut c).await {
            Ok(_) => "ok",
            Err(_) => "error",
        },
    };
    let status = if db == "error" || redis == "error" {
        "degraded"
    } else {
        "ok"
    };
    Json(Health {
        status,
        version: env!("CARGO_PKG_VERSION"),
        env: s.0.config.app_env.clone(),
        db,
        redis,
        table_prefix: s.0.config.prefix.as_str().to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn app() -> Router {
        let cfg = Config {
            app_env: "test".into(),
            addr: ([127, 0, 0, 1], 0).into(),
            database_url: None,
            redis_url: None,
            prefix: Default::default(),
            data_path: ".".into(),
            cors_origin: None,
        };
        router(AppState::without_backends(cfg))
    }

    async fn call(uri: &str) -> (u16, serde_json::Value) {
        let res = app()
            .oneshot(Request::get(uri).body(Body::empty()).unwrap_or_default())
            .await
            .unwrap_or_else(|e| match e {});
        let status = res.status().as_u16();
        let bytes = res
            .into_body()
            .collect()
            .await
            .map(|b| b.to_bytes())
            .unwrap_or_default();
        (status, serde_json::from_slice(&bytes).unwrap_or_default())
    }

    #[tokio::test]
    async fn health_ok_without_backends() {
        let (status, body) = call("/api/v1/health").await;
        assert_eq!(status, 200);
        assert_eq!(body["status"], "ok");
        assert_eq!(body["db"], "disabled");
        assert_eq!(body["tablePrefix"], "g5_");
    }

    #[tokio::test]
    async fn unknown_path_is_json_404() {
        let (status, body) = call("/api/v1/nope").await;
        assert_eq!(status, 404);
        assert_eq!(body["error"]["code"], "NOT_FOUND");
    }
}
