use crate::server::actuator_response::ActuatorResponse;
use crate::server::app_state::AppState;
use crate::server::error::AppError;
use axum::Json;
use axum::extract::State;
use color_eyre::eyre::OptionExt;
use convertor::config::client_config::ProxyClient;
use convertor::config::provider_config::Provider;
use redis::AsyncTypedCommands;
use serde_json::json;
use std::sync::Arc;
use strum::VariantArray;

pub type AnyJsonResponse = Json<ActuatorResponse<serde_json::Value>>;

pub async fn healthy() -> Json<ActuatorResponse<()>> {
    Json(ActuatorResponse::<()>::ok())
}

pub async fn redis(State(state): State<Arc<AppState>>) -> Result<AnyJsonResponse, AppError> {
    let pong = state
        .redis_connection
        .clone()
        .ok_or_eyre("没有 Redis 连接")?
        .ping()
        .await?;
    Ok(Json(ActuatorResponse::ok_data(json!({
        "pong": pong
    }))))
}

pub async fn version() -> Result<Json<ActuatorResponse<serde_json::Value>>, AppError> {
    Ok(Json(ActuatorResponse::ok_data(json!({
        "clients": ProxyClient::VARIANTS,
        "providers": Provider::VARIANTS,
        "version": env!("CARGO_PKG_VERSION").to_string()
    }))))
}
