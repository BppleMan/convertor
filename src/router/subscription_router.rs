use crate::error::AppError;
use crate::router::AppState;
use crate::router::query::SubLogQuery;
use crate::service_provider::api::subscription_log::SubscriptionLog;
use axum::Json;
use axum::extract::{RawQuery, State};
use color_eyre::eyre::{OptionExt, eyre};
use std::sync::Arc;

pub async fn subscription_logs(
    State(state): State<Arc<AppState>>,
    RawQuery(query): RawQuery,
) -> Result<Json<Vec<SubscriptionLog>>, AppError> {
    let query = query.as_ref().ok_or_eyre(eyre!("订阅记录必须传递参数"))?;
    let mut sub_log_query = SubLogQuery::decode_from_query_string(query, &state.config.secret)?;
    if sub_log_query.secret != state.config.secret {
        return Err(AppError::Unauthorized("Invalid secret".to_string()));
    }
    let logs = state.api.get_sub_logs().await?;
    let logs = if let (Some(current), Some(size)) = (sub_log_query.page_current.take(), sub_log_query.page_size.take())
    {
        let start = (current - 1) * size;
        logs.0.into_iter().skip(start).take(size).collect()
    } else {
        logs.0
    };
    Ok(Json(logs))
}
