use color_eyre::eyre::eyre;
use color_eyre::Result;
use std::process::Stdio;
use tokio::process::Command;

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
