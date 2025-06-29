use crate::config::convertor_config::ConvertorConfig;
use crate::error::AppError;
use crate::subscription::subscription_api::boslife_api::BosLifeApi;
use color_eyre::eyre::eyre;
use color_eyre::Result;

pub mod clash;
pub mod surge;
pub mod subscription;

pub struct AppState {
    pub convertor_config: ConvertorConfig,
    pub subscription_api: BosLifeApi,
}

pub async fn root() -> Result<(), AppError> {
    Err(eyre!("Hello, World!"))?;
    Ok(())
}
