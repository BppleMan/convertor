use redis::{Client, TlsCertificates};
use std::sync::OnceLock;

pub static REDIS_ENDPOINT: OnceLock<String> = OnceLock::new();
pub static REDIS_CONVERTOR_USERNAME: OnceLock<String> = OnceLock::new();
pub static REDIS_CONVERTOR_PASSWORD: OnceLock<String> = OnceLock::new();
pub const REDIS_CONVERTOR_CONFIG_KEY: &str = "convertor:config.toml";
pub const REDIS_CONVERTOR_CONFIG_PUBLISH_CHANNEL: &str = "convertor:config:publish";

pub fn init_redis_info() -> color_eyre::Result<()> {
    let endpoint = std::env::var("REDIS_ENDPOINT")?;
    let username = std::env::var("REDIS_CONVERTOR_USERNAME")?;
    let password = std::env::var("REDIS_CONVERTOR_PASSWORD")?;
    REDIS_ENDPOINT.get_or_init(|| endpoint);
    REDIS_CONVERTOR_USERNAME.get_or_init(|| username);
    REDIS_CONVERTOR_PASSWORD.get_or_init(|| password);
    Ok(())
}

pub fn redis_url() -> String {
    #[cfg(not(debug_assertions))]
    return format!(
        "rediss://{}:{}@{}/0?protocol=resp3",
        REDIS_CONVERTOR_USERNAME
            .get()
            .expect("REDIS_CONVERTOR_USERNAME not set"),
        REDIS_CONVERTOR_PASSWORD
            .get()
            .expect("REDIS_CONVERTOR_PASSWORD not set"),
        REDIS_ENDPOINT.get().expect("REDIS_ENDPOINT not set")
    );
    #[cfg(debug_assertions)]
    return format!(
        "rediss://{}:{}@{}/1?protocol=resp3",
        REDIS_CONVERTOR_USERNAME.get().expect("CONFIG_CENTER_USERNAME not set"),
        REDIS_CONVERTOR_PASSWORD.get().expect("CONFIG_CENTER_PASSWORD not set"),
        REDIS_ENDPOINT.get().expect("CONFIG_CENTER_ENDPOINT not set")
    );
}

pub fn redis_client(redis_url: impl AsRef<str>) -> color_eyre::Result<Client> {
    let ca_cert = std::env::var("REDIS_CA_CERT")?;
    let client = Client::build_with_tls(
        redis_url.as_ref(),
        TlsCertificates {
            client_tls: None,
            root_cert: Some(ca_cert.into_bytes()),
        },
    )?;
    Ok(client)
}
