use crate::common::url::ConvertorUrl;
use crate::core::error::ParseError;
use crate::core::profile::Profile;
use crate::core::profile::clash_profile::ClashProfile;
use crate::core::profile::policy::Policy;
use crate::core::renderer::Renderer;
use crate::core::renderer::clash_renderer::ClashRenderer;
use crate::server::AppState;
use color_eyre::Result;
use std::sync::Arc;
use tracing::instrument;

#[instrument(skip_all)]
pub async fn profile_impl(state: Arc<AppState>, url: ConvertorUrl, raw_profile: String) -> Result<String> {
    let profile = try_get_profile(state, url, raw_profile).await?;
    Ok(ClashRenderer::render_profile(&profile)?)
}

#[instrument(skip_all)]
pub async fn rule_provider_impl(
    state: Arc<AppState>,
    url: ConvertorUrl,
    raw_profile: String,
    policy: Policy,
) -> Result<String> {
    let profile = try_get_profile(state, url, raw_profile).await?;
    match profile.get_provider_rules_with_policy(&policy) {
        None => Ok(String::new()),
        Some(provider_rules) => Ok(ClashRenderer::render_provider_rules(provider_rules)?),
    }
}

async fn try_get_profile(state: Arc<AppState>, url: ConvertorUrl, raw_profile: String) -> Result<ClashProfile> {
    let profile = state
        .clash_cache
        .try_get_with(url.clone(), async {
            let profile = ClashProfile::parse(raw_profile)?;
            let mut template = ClashProfile::template()?;
            template.merge(profile)?;
            template.convert(&url).map_err(Arc::new)?;
            Ok::<_, Arc<ParseError>>(template)
        })
        .await?;
    Ok(profile)
}

#[cfg(test)]
mod clash_test {
    use crate::common::proxy_client::ProxyClient;
    use crate::core::profile::policy::Policy;
    use crate::server::server_mock::{ServerContext, expect_profile, expect_rule_provider, start_server};
    use axum::body::Body;
    use axum::extract::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    pub async fn test_clash_profile() -> color_eyre::Result<()> {
        let ServerContext { app, app_state } = start_server(ProxyClient::Clash).await?;
        let convertor_url = app_state.config.create_convertor_url(ProxyClient::Clash)?;

        let url = convertor_url.build_sub_url()?;
        let uri = format!("{}?{}", url.path(), url.query().expect("必须有查询参数"));
        let request = Request::builder()
            .uri(uri)
            .header("host", app_state.config.server_addr()?)
            .method("GET")
            .body(Body::empty())?;
        let response = app.oneshot(request).await?;

        let actual = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).to_string();
        let expect = expect_profile(ProxyClient::Clash, convertor_url.encoded_raw_sub_url()?);
        pretty_assertions::assert_str_eq!(expect, actual);

        Ok(())
    }

    #[tokio::test]
    pub async fn test_clash_boslife_policy_provider() -> color_eyre::Result<()> {
        let ServerContext { app, app_state } = start_server(ProxyClient::Clash).await?;
        let convertor_url = app_state.config.create_convertor_url(ProxyClient::Clash)?;
        let policy = Policy {
            name: "BosLife".to_string(),
            option: None,
            is_subscription: false,
        };

        let url = convertor_url.build_rule_provider_url(&policy)?;
        let uri = format!("{}?{}", url.path(), url.query().expect("必须有查询参数"));
        let request = Request::builder()
            .uri(uri)
            .header("host", app_state.config.server_addr()?)
            .method("GET")
            .body(Body::empty())?;
        let response = app.oneshot(request).await?;

        let actual = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).to_string();
        let expect = expect_rule_provider(ProxyClient::Clash, &policy);
        pretty_assertions::assert_str_eq!(expect, actual);

        Ok(())
    }

    #[tokio::test]
    pub async fn test_clash_subscription_rule_provider() -> color_eyre::Result<()> {
        let ServerContext { app, app_state } = start_server(ProxyClient::Clash).await?;
        let convertor_url = app_state.config.create_convertor_url(ProxyClient::Clash)?;
        let policy = Policy::subscription_policy();

        let url = convertor_url.build_rule_provider_url(&policy)?;
        let uri = format!("{}?{}", url.path(), url.query().expect("必须有查询参数"));
        let request = Request::builder()
            .uri(uri)
            .header("host", app_state.config.server_addr()?)
            .method("GET")
            .body(Body::empty())?;
        let response = app.oneshot(request).await?;

        let actual = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).to_string();
        pretty_assertions::assert_str_eq!(
            format!("payload:\n    - DOMAIN,http://{}", convertor_url.raw_sub_host()?),
            actual.trim()
        );

        Ok(())
    }

    #[tokio::test]
    pub async fn test_clash_direct_rule_provider() -> color_eyre::Result<()> {
        let ServerContext { app, app_state } = start_server(ProxyClient::Clash).await?;
        let convertor_url = app_state.config.create_convertor_url(ProxyClient::Clash)?;
        let policy = Policy::direct_policy();
        let url = convertor_url.build_rule_provider_url(&policy)?;

        let uri = format!("{}?{}", url.path(), url.query().expect("必须有查询参数"));
        let request = Request::builder()
            .uri(uri)
            .header("host", app_state.config.server_addr()?)
            .method("GET")
            .body(Body::empty())?;
        let response = app.oneshot(request).await?;

        let actual = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).to_string();
        let expect = expect_rule_provider(ProxyClient::Clash, &policy);
        pretty_assertions::assert_str_eq!(expect, actual);

        Ok(())
    }
}
