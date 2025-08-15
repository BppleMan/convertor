use convertor::common::redis_info::{REDIS_CONVERTOR_CONFIG_KEY, init_redis_info, redis_client, redis_url};
use redis::AsyncTypedCommands;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    init_redis_info()?;
    println!("{}", redis_url());
    let client = redis_client(redis_url())?;
    let mut connection = client.get_multiplexed_async_connection().await?;
    let config = connection.get(REDIS_CONVERTOR_CONFIG_KEY).await?;
    println!("Config: {config:?}");
    Ok(())
}
