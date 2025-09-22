use crate::server::app_state::AppState;
use crate::server::response::{ApiError, ApiResponse};
use axum::extract::State;
use color_eyre::eyre::OptionExt;
use redis::AsyncTypedCommands;
use serde_json::json;
use std::sync::Arc;
use tracing::instrument;

#[instrument(skip_all)]
pub async fn healthy() -> ApiResponse<()> {
    ApiResponse::ok(())
}

#[instrument(skip_all)]
pub async fn redis(State(state): State<Arc<AppState>>) -> Result<ApiResponse<String>, ApiError> {
    let pong = state
        .redis_connection
        .clone()
        .ok_or_eyre("没有 Redis 连接")?
        .ping()
        .await?;
    Ok(ApiResponse::ok(pong))
}
