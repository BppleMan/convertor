use color_eyre::eyre::eyre;
use color_eyre::Result;
use std::process::Stdio;
use tokio::process::Command;
use tracing::info;

#[derive(Debug)]
pub struct OpItem {
    pub username: String,
    pub password: String,
}

pub async fn get_item(name: &str) -> Result<OpItem> {
    let output = Command::new("op")
        .arg("item")
        .arg("get")
        .arg(name)
        .arg("--fields")
        .arg("username,password")
        .arg("--reveal")
        .stdout(Stdio::piped())
        .output()
        .await?;

    if output.status.success() {
        let content = String::from_utf8(output.stdout)?;
        let (username, password) = content
            .trim()
            .split_once(',')
            .map(|(u, p)| (u.to_string(), p.to_string()))
            .ok_or_else(|| eyre!("op get item failed: invalid content"))?;
        Ok(OpItem { username, password })
    } else {
        Err(eyre!(
            "op get item failed: {}",
            String::from_utf8(output.stderr)?
        ))
    }
}

pub fn get_convertor_secret() -> Result<String> {
    info!("尝试从环境变量中获取 $CONVERTOR_SECRET");
    let secret = std::env::var("CONVERTOR_SECRET").ok();
    if let Some(secret) = secret {
        return Ok(secret);
    }
    info!("尝试从 1Password 中获取 CONVERTOR_SECRET");
    tokio::runtime::Handle::current().block_on(async {
        let output = Command::new("op")
            .arg("item")
            .arg("get")
            .arg("CONVERTOR_SECRET")
            .arg("--fields")
            .arg("password")
            .arg("--reveal")
            .stdout(Stdio::piped())
            .output()
            .await?;

        if output.status.success() {
            let secret = String::from_utf8(output.stdout)?.trim().to_string();
            Ok(secret)
        } else {
            Err(eyre!(
                "op get convertor_secret failed: {}",
                String::from_utf8(output.stderr)?
            ))
        }
    })
}
