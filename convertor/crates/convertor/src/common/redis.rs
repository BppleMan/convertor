use redis::Client;
use std::sync::OnceLock;
use tracing::debug;

pub static REDIS_ENDPOINT: OnceLock<String> = OnceLock::new();
pub static REDIS_SCHEME: OnceLock<String> = OnceLock::new();
pub static REDIS_CONVERTOR_USERNAME: OnceLock<String> = OnceLock::new();
pub static REDIS_CONVERTOR_PASSWORD: OnceLock<String> = OnceLock::new();
pub const REDIS_CONVERTOR_CONFIG_KEY: &str = "convertor:config.toml";
pub const REDIS_CONVERTOR_CONFIG_PUBLISH_CHANNEL: &str = "convertor:config:publish";

pub fn init_redis() -> color_eyre::Result<()> {
    let endpoint = std::env::var("REDIS_ENDPOINT").expect("REDIS_ENDPOINT not set");
    let scheme = std::env::var("REDIS_SCHEME").expect("REDIS_SCHEME not set");
    let username = std::env::var("REDIS_CONVERTOR_USERNAME").expect("REDIS_CONVERTOR_USERNAME not set");
    let password = std::env::var("REDIS_CONVERTOR_PASSWORD").expect("REDIS_CONVERTOR_PASSWORD not set");
    #[cfg(debug_assertions)]
    {
        println!("Redis Endpoint: {endpoint}");
        println!("Redis Scheme: {scheme}");
        println!("Redis Username: {username}");
        println!("Redis Password: {password}");
    }
    REDIS_ENDPOINT.get_or_init(|| endpoint);
    REDIS_SCHEME.get_or_init(|| scheme);
    REDIS_CONVERTOR_USERNAME.get_or_init(|| username);
    REDIS_CONVERTOR_PASSWORD.get_or_init(|| password);
    Ok(())
}

pub fn redis_url() -> String {
    #[cfg(not(debug_assertions))]
    let database = "0";
    #[cfg(debug_assertions)]
    let database = "1";
    format!(
        "{}://{}:{}@{}/{database}?protocol=resp3",
        REDIS_SCHEME.get().expect("REDIS_SCHEME not set"),
        REDIS_CONVERTOR_USERNAME
            .get()
            .expect("REDIS_CONVERTOR_USERNAME not set"),
        REDIS_CONVERTOR_PASSWORD
            .get()
            .expect("REDIS_CONVERTOR_PASSWORD not set"),
        REDIS_ENDPOINT.get().expect("REDIS_ENDPOINT not set")
    )
}

pub fn redis_client(redis_url: impl AsRef<str>) -> color_eyre::Result<Client> {
    debug!("Redis URL: {}", redis_url.as_ref());
    let ca_cert = std::env::var("REDIS_CA_CERT");
    match (REDIS_SCHEME.get().as_ref().map(|s| s.as_str()), ca_cert) {
        (Some("redis"), _) => {
            let client = Client::open(redis_url.as_ref())?;
            Ok(client)
        }
        (Some("rediss"), Ok(ca_cert)) => {
            #[cfg(debug_assertions)]
            {
                // 校验 ca cert 格式
                if !ca_cert.contains("BEGIN CERTIFICATE") || !ca_cert.contains("END CERTIFICATE") {
                    panic!("REDIS_CA_CERT 格式错误, 必须包含 -----BEGIN CERTIFICATE----- 和 -----END CERTIFICATE-----");
                } else {
                    debug!("Redis CA Cert: 已加载");
                }
            }
            let client = Client::build_with_tls(
                redis_url.as_ref(),
                redis::TlsCertificates {
                    client_tls: None,
                    root_cert: Some(ca_cert.into_bytes()),
                },
            )?;
            Ok(client)
        }
        (Some("rediss"), Err(e)) => Err(e.into()),
        _ => {
            Err(color_eyre::eyre::eyre!(
                "REDIS_SCHEME must be set to 'redis' or 'rediss'"
            ))
        }
    }
}
