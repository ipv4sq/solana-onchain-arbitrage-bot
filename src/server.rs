use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Router};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::config::Config;

#[derive(Clone)]
struct AppState {
    config: Arc<Config>,
}

pub async fn run_server(config: Config) -> anyhow::Result<()> {
    let state = AppState {
        config: Arc::new(config),
    };

    let app = Router::new()
        .route("/config", get(get_config))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = "0.0.0.0:8080";
    tracing::info!("HTTP server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn get_config(State(state): State<AppState>) -> impl IntoResponse {
    match toml::to_string_pretty(&*state.config) {
        Ok(toml_string) => (StatusCode::OK, toml_string),
        Err(e) => {
            tracing::error!("Failed to serialize config: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to serialize configuration".to_string(),
            )
        }
    }
}
