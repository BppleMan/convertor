use axum::extract::FromRequest;
use axum::{async_trait, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct ApiKey {
    secret: String,
}

#[async_trait]
impl<S> FromRequest<S> for ApiKey
where
    S: Send, // 必须的，因为 FromRequest 是异步的
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        if let Some(query) = req.uri().query() {
            let params: HashMap<String, String> =
                serde_qs::from_str(query).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid query string"))?;
            if let Some(secret) = params.get("secret") {
                Ok(ApiKey {
                    secret: secret.to_string(),
                })
            } else if let Some(auth) = req.headers().get("Secret") {
                Ok(ApiKey {
                    secret: auth
                        .to_str()
                        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid header"))?
                        .to_string(),
                })
            } else {
                Err((StatusCode::UNAUTHORIZED, "Invalid API key"))
            }
        } else {
            Err((StatusCode::UNAUTHORIZED, "Invalid API key"))
        }
    }
}
