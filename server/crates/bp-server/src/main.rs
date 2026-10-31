//! beripickboard API 서버.

mod auth;
mod config;
mod error;
mod g5;
mod routes;
mod state;

use anyhow::Result;
use axum::http::{HeaderValue, Method};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,bp_server=debug".into()))
        .init();

    let config = config::Config::from_env()?;
    let addr = config.addr;
    tracing::info!(data_path = %config.data_path, prefix = config.prefix.as_str(), "설정 로드");
    let cors = match &config.cors_origin {
        Some(o) => CorsLayer::new()
            .allow_origin(o.parse::<HeaderValue>()?)
            .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
            .allow_headers([
                axum::http::header::CONTENT_TYPE,
                axum::http::header::AUTHORIZATION,
            ])
            .allow_credentials(true),
        None => CorsLayer::new(),
    };
    let state = state::AppState::new(config).await?;
    let app = routes::router(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, version = env!("CARGO_PKG_VERSION"), "beripickboard 서버 시작");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown())
    .await?;
    Ok(())
}

async fn shutdown() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("종료 신호 수신");
}
