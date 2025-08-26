use crate::server::app_state::AppState;
use crate::server::error::AppError;
use axum::Json;
use axum::extract::State;
use color_eyre::eyre::OptionExt;
use redis::AsyncTypedCommands;
use std::collections::HashMap;
use std::sync::Arc;

pub async fn redis(State(state): State<Arc<AppState>>) -> Result<Json<HashMap<String, String>>, AppError> {
    let pong = state
        .redis_connection
        .clone()
        .ok_or_eyre("没有 Redis 连接")?
        .ping()
        .await?;
    let mut result = HashMap::new();
    result.insert("pong".to_string(), pong);
    Ok(Json(result))
}

pub async fn version() -> Result<Json<HashMap<String, String>>, AppError> {
    let mut result = HashMap::new();
    result.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
    Ok(Json(result))
}
