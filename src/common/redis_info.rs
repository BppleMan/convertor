use std::sync::OnceLock;

pub static CONFIG_CENTER_ENDPOINT: OnceLock<String> = OnceLock::new();
pub static CONFIG_CENTER_USERNAME: OnceLock<String> = OnceLock::new();
pub static CONFIG_CENTER_PASSWORD: OnceLock<String> = OnceLock::new();
pub static CONFIG_CENTER_CONVERTOR_CONFIG_KEY: &str = "convertor:config.toml";
pub const CONFIG_CENTER_CONVERTOR_CONFIG_PUBLISH_CHANNEL: &str = "convertor:config:publish";

pub fn init_redis_info() -> color_eyre::Result<()> {
    let endpoint = std::env::var("CONFIG_CENTER_ENDPOINT")?;
    let username = std::env::var("CONFIG_CENTER_USERNAME")?;
    let password = std::env::var("CONFIG_CENTER_PASSWORD")?;
    CONFIG_CENTER_ENDPOINT.get_or_init(|| endpoint);
    CONFIG_CENTER_USERNAME.get_or_init(|| username);
    CONFIG_CENTER_PASSWORD.get_or_init(|| password);
    Ok(())
}

pub fn config_center_url() -> String {
    format!(
        "redis://{}:{}@{}?protocol=resp3",
        CONFIG_CENTER_USERNAME.get().expect("CONFIG_CENTER_USERNAME not set"),
        CONFIG_CENTER_PASSWORD.get().expect("CONFIG_CENTER_PASSWORD not set"),
        CONFIG_CENTER_ENDPOINT.get().expect("CONFIG_CENTER_ENDPOINT not set")
    )
}

// #[derive(Debug, Clone, Copy)]
// pub enum RedisPublishChannel {
//     ConfigCenterConvertorConfigPublish,
// }
//
// impl FromStr for RedisPublishChannel {
//     type Err = ();
//
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s {
//             CONFIG_CENTER_CONVERTOR_CONFIG_PUBLISH_CHANNEL => {
//                 Ok(RedisPublishChannel::ConfigCenterConvertorConfigPublish)
//             }
//             _ => Err(()),
//         }
//     }
// }
