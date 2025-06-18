use crate::error::AppError;
use crate::server::route::AppState;
use crate::service::service_api::ServiceApi;
use crate::service::subscription_log::SubscriptionLog;
use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionQuery {
    pub page_current: Option<usize>,
    pub page_size: Option<usize>,
}

pub async fn subscription_log(
    State(state): State<Arc<AppState>>,
    mut query: Query<SubscriptionQuery>,
) -> Result<Json<Vec<SubscriptionLog>>, AppError> {
    let auth_token = state.service.login().await?;
    let mut logs: Vec<SubscriptionLog> =
        state.service.get_subscription_log(&auth_token).await?;
    if let (Some(current), Some(size)) =
        (query.page_current.take(), query.page_size.take())
    {
        let start = (current - 1) * size;
        logs = logs.into_iter().skip(start).take(size).collect();
    }
    Ok(Json(logs))
}
