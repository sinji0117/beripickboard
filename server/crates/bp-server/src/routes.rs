use crate::{auth::handlers as auth, board::handlers as board, error::AppError, state::AppState};
use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use serde::Serialize;

pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/health", get(health))
        .route("/site", get(site))
        .route("/auth/login", post(auth::login))
        .route("/auth/refresh", post(auth::refresh_token))
        .route("/auth/logout", post(auth::logout))
        .route("/members/me", get(auth::me))
        .route("/boards", get(board::list_boards))
        .route("/boards/{bo_table}/posts", get(board::list_posts))
        .route("/boards/{bo_table}/posts/{wr_id}", get(board::view_post))
        .route(
            "/boards/{bo_table}/posts/{wr_id}/unlock",
            post(board::unlock_post),
        )
        .route(
            "/boards/{bo_table}/posts/{wr_id}/files/{no}",
            get(board::download),
        )
        .route(
            "/boards/{bo_table}/posts/{wr_id}/files/{no}/inline",
            get(board::inline_image),
        );
    Router::new()
        .nest("/api/v1", api)
        .fallback(|| async { AppError::not_found("NOT_FOUND", "요청한 경로가 없습니다.") })
        .with_state(state)
}

/// GET /api/v1/site — 사이트 공개 정보 (g5_config)
async fn site(State(s): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let db =
        s.0.db
            .as_ref()
            .ok_or_else(|| AppError::unavailable("DB_DISABLED", "DB가 설정되지 않았습니다."))?;
    let cfg = crate::g5::config::load(db, &s.0.config.prefix).await?;
    Ok(Json(
        serde_json::json!({ "data": { "title": cfg.cf_title, "theme": cfg.cf_theme } }),
    ))
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
        router(AppState::without_backends(Config::for_test()))
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
