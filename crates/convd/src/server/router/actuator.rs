use crate::server::app_state::AppState;
use crate::server::response::{ApiError, ApiResponse};
use axum::extract::State;
use redis::AsyncTypedCommands;
use std::sync::Arc;
use tracing::instrument;

#[instrument(skip_all)]
pub async fn healthy() -> ApiResponse<()> {
    ApiResponse::ok(())
}

#[instrument(skip_all)]
pub async fn redis(State(state): State<Arc<AppState>>) -> Result<ApiResponse<String>, ApiError> {
    let pong = async move {
        let pong = state.redis_connection.clone()?.ping().await;
        Some(pong)
    }
    .await
    .transpose()
    .map_err(ApiError::internal_server_error)?;
    Ok(ApiResponse::ok(pong.unwrap_or_else(|| "Redis not configured".to_string())))
}

#[instrument(skip_all)]
pub async fn metrics() -> ApiResponse<()> {
    ApiResponse::ok(())
}
