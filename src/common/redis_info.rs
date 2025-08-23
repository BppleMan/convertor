use redis::Client;
use std::sync::OnceLock;

pub static REDIS_ENDPOINT: OnceLock<String> = OnceLock::new();
pub static REDIS_CONVERTOR_USERNAME: OnceLock<String> = OnceLock::new();
pub static REDIS_CONVERTOR_PASSWORD: OnceLock<String> = OnceLock::new();
pub const REDIS_CONVERTOR_CONFIG_KEY: &str = "convertor:config.toml";
pub const REDIS_CONVERTOR_CONFIG_PUBLISH_CHANNEL: &str = "convertor:config:publish";

pub fn init_redis_info() -> color_eyre::Result<()> {
    let endpoint = std::env::var("REDIS_ENDPOINT").expect("REDIS_ENDPOINT not set");
    let username = std::env::var("REDIS_CONVERTOR_USERNAME").expect("REDIS_CONVERTOR_USERNAME not set");
    let password = std::env::var("REDIS_CONVERTOR_PASSWORD").expect("REDIS_CONVERTOR_PASSWORD not set");
    #[cfg(debug_assertions)]
    {
        println!("Redis Endpoint: {endpoint}");
        println!("Redis Username: {username}");
        println!("Redis Password: {password}");
    }
    REDIS_ENDPOINT.get_or_init(|| endpoint);
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
        "rediss://{}:{}@{}/{database}?protocol=resp3",
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
    println!("Redis URL: {}", redis_url.as_ref());
    let ca_cert = std::env::var("REDIS_CA_CERT")?;
    #[cfg(debug_assertions)]
    println!("Redis CA Cert: {ca_cert}");
    let client = Client::build_with_tls(
        redis_url.as_ref(),
        redis::TlsCertificates {
            client_tls: None,
            root_cert: Some(ca_cert.into_bytes()),
        },
    )?;
    // let client = Client::open(redis_url.as_ref())?;
    Ok(client)
}
