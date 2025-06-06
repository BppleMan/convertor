use crate::convertor_url::ConvertorUrl;
use crate::error::AppError;
use crate::service::boslife_service::BosLifeService;
use axum::body::Body;
use axum::http::{header, Request};
use color_eyre::eyre::eyre;
use color_eyre::{Report, Result};

pub mod clash;
pub mod surge;
pub mod subscription;

pub struct AppState {
    pub service: BosLifeService,
}

pub async fn root() -> Result<(), AppError> {
    Err(eyre!("Hello, World!"))?;
    Ok(())
}

pub fn extract_convertor_url(
    request: &Request<Body>,
) -> Result<ConvertorUrl, Report> {
    let host = request
        .headers()
        .get(header::HOST)
        .ok_or_else(|| eyre!("Missing Host header"))?
        .to_str()?;
    let full_url = format!("http://{}{}", host, request.uri());
    ConvertorUrl::decode_from_convertor_url(full_url)
}
