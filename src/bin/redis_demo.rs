use convertor::common::config::ConvertorConfig;
use convertor::common::redis_info::CONFIG_CENTER_CONVERTOR_CONFIG_PUBLISH_CHANNEL;
use redis::AsyncCommands;
use std::str::FromStr;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let endpoint = env!("CONFIG_CENTER_ENDPOINT");
    let username = env!("CONFIG_CENTER_USERNAME");
    let password = env!("CONFIG_CENTER_PASSWORD");
    let config_key = env!("CONFIG_CENTER_CONVERTOR_CONFIG_KEY");
    let client = redis::Client::open(format!("redis://{username}:{password}@{endpoint}?protocol=resp3"))?;
    let mut con = client.get_multiplexed_tokio_connection().await?;
    let config: Option<String> = con.get(config_key).await?;
    let config = ConvertorConfig::from_str(&config.unwrap())?;
    println!("{config:#?}");
    client.get_multiplexed_tokio_connection().await?;
    let mut pubsub = client.get_async_pubsub().await?;
    pubsub.subscribe(CONFIG_CENTER_CONVERTOR_CONFIG_PUBLISH_CHANNEL).await?;
    let mut pubsub_stream = pubsub.into_on_message();
    while let Some(message) = pubsub_stream.next().await {
        println!(
            "{}: {}",
            message.get_channel_name(),
            str::from_utf8(message.get_payload_bytes())?
        );
    }
    Ok(())
}
