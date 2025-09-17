use crate::server::app_state::AppState;
use crate::server::response::{ApiError, ApiResponse};
use axum::extract::State;
use color_eyre::eyre::OptionExt;
use convertor::config::client_config::ProxyClient;
use convertor::config::provider_config::Provider;
use redis::AsyncTypedCommands;
use serde_json::json;
use std::sync::Arc;
use strum::VariantArray;

pub async fn healthy() -> ApiResponse<()> {
    ApiResponse::ok(())
}

pub async fn redis(State(state): State<Arc<AppState>>) -> Result<ApiResponse<String>, ApiError> {
    let pong = state
        .redis_connection
        .clone()
        .ok_or_eyre("没有 Redis 连接")?
        .ping()
        .await?;
    Ok(ApiResponse::ok(pong))
}

pub async fn version() -> Result<ApiResponse<serde_json::Value>, ApiError> {
    Ok(ApiResponse::ok(json!({
        "clients": ProxyClient::VARIANTS,
        "providers": Provider::VARIANTS,
        "version": env!("CARGO_PKG_VERSION").to_string()
    })))
}
