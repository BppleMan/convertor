use crate::encrypt::decrypt;
use crate::error::AppError;
use crate::router::AppState;
use crate::router::query::SubLogQuery;
use crate::subscription::subscription_log::SubscriptionLog;
use axum::Json;
use axum::extract::{Query, State};
use percent_encoding::percent_decode_str;
use std::sync::Arc;

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
