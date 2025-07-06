use crate::encrypt::decrypt;
use crate::error::AppError;
use crate::server::router::AppState;
use crate::subscription::subscription_log::SubscriptionLog;
use axum::extract::{Query, State};
use axum::Json;
use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct SubLogQuery {
    pub secret: String,
    pub page_current: Option<usize>,
    pub page_size: Option<usize>,
}

pub async fn subscription_logs(
    State(state): State<Arc<AppState>>,
    mut query: Query<SubLogQuery>,
) -> Result<Json<Vec<SubscriptionLog>>, AppError> {
    let encrypted_secret = percent_decode_str(&query.secret).decode_utf8()?;
    let decrypted_secret = decrypt(state.config.secret.as_bytes(), &encrypted_secret)?;
    if decrypted_secret != state.config.secret {
        return Err(AppError::Unauthorized("Invalid secret".to_string()));
    }
    let mut logs = state
        .api
        .get_sub_logs(state.config.service_config.base_url.clone())
        .await?;
    if let (Some(current), Some(size)) = (query.page_current.take(), query.page_size.take()) {
        let start = (current - 1) * size;
        logs = logs.into_iter().skip(start).take(size).collect();
    }
    Ok(Json(logs))
}
