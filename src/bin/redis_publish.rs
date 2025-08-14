use redis::AsyncCommands;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let endpoint = env!("CONFIG_CENTER_ENDPOINT");
    let username = env!("CONFIG_CENTER_USERNAME");
    let password = env!("CONFIG_CENTER_PASSWORD");
    let client = redis::Client::open(format!("redis://{username}:{password}@{endpoint}?protocol=resp3"))?;
    let mut con = client.get_multiplexed_tokio_connection().await?;
    let count: usize = AsyncCommands::publish(&mut con, "convertor_config_update", "").await?;
    println!("Published {count} messages to 'convertor_config_update'");
    Ok(())
}
