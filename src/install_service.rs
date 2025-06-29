use crate::subscription::subscription_api::boslife_api::BosLifeApi;
use crate::subscription::subscription_api::ServiceApi;
use color_eyre::eyre::{eyre, WrapErr};
use flate2::bufread::GzDecoder;
use futures_util::StreamExt;
use inquire::Confirm;
use reqwest::{Client, Method, StatusCode};
use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;
use url::Url;

const SYSTEMD_DIR_STR: &str = "/etc/systemd/system";

const CONVERTOR_SERVICE_TEMPLATE: &[u8] = include_bytes!("../assets/service/convertor.service");
const MIHOMO_SERVICE_TEMPLATE: &[u8] = include_bytes!("../assets/service/mihomo.service");

pub async fn install_service(name: impl AsRef<str>, service_api: BosLifeApi) -> color_eyre::Result<()> {
    let name = name.as_ref();

    let template = match name {
        "convertor" => CONVERTOR_SERVICE_TEMPLATE,
        "mihomo" => {
            download_mihomo(service_api.client()).await?;
            MIHOMO_SERVICE_TEMPLATE
        }
        _ => return Err(eyre!("不支持安装该服务: {}", name)),
    };

    let status = copy_service_file(name, template)?;
    if !status {
        println!("跳过拷贝 systemd 配置: {}", name);
        return Ok(());
    };

    load_service(name)?;

    start_service(name)?;

    println!("服务: {} 安装成功", name);
    Ok(())
}

async fn download_mihomo(client: &Client) -> color_eyre::Result<()> {
    println!("正在下载最新版本的 mihomo...");
    let latest_url = "https://github.com/MetaCubeX/mihomo/releases/latest/download";
    let version_txt_url = Url::parse(&format!("{}/version.txt", latest_url))?;
    let http_response = client
        .request(Method::GET, version_txt_url)
        .send()
        .await?
        .error_for_status()?;
    if http_response.status() == StatusCode::OK {
        let version = http_response.text().await?;
        println!("最新版本: {}", version.trim());
        let mihomo_url = format!("{}/mihomo-linux-amd64-{}.gz", latest_url, version);
        let http_response = client
            .request(Method::GET, &mihomo_url)
            .send()
            .await?
            .error_for_status()?;
        let mut downloaded_size = 0u64;
        let total_size = http_response.content_length().unwrap_or(0u64);
        if total_size == 0 {
            return Err(eyre!("下载的文件大小为 0，可能是版本信息错误"));
        }
        let mut stream = http_response.bytes_stream();
        let mut compressed = vec![];
        while let Some(Ok(chunk)) = stream.next().await {
            downloaded_size += chunk.len() as u64;
            compressed.extend(chunk);
            println!("下载中... {}/{} bytes", downloaded_size, total_size);
        }
        if downloaded_size != total_size {
            return Err(eyre!(
                "下载的文件大小不匹配，期望: {}, 实际: {}",
                total_size,
                downloaded_size
            ));
        }
        println!("下载完成，大小: {} bytes", downloaded_size);
        let mut decoder = GzDecoder::new(&compressed[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        println!(
            "解压缩完成，大小: {} bytes，写入到 /usr/local/bin/mihomo",
            decompressed.len()
        );
        let mihomo_path = Path::new("/usr/local/bin/mihomo");
        let mut mihomo_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(mihomo_path)?;
        mihomo_file.write_all(&decompressed)?;
        mihomo_file.flush()?;
        Command::new("chmod")
            .arg("+x")
            .arg(mihomo_path)
            .spawn()
            .wrap_err_with(|| format!("无法设置可执行权限: {}", mihomo_path.display()))?
            .wait()
            .wrap_err_with(|| "无法等待 chmod 命令执行完成")?;
        println!("mihomo 已写入到: {}", mihomo_path.display());
    } else {
        return Err(eyre!("无法获取最新版本信息，状态码: {}", http_response.status()));
    }
    Ok(())
    // let mihomo_url = "https://objects.githubusercontent.com/github-production-release-asset-2e65be/369178935/4673aec0-3c70-4def-8276-ddcfe8ad0a2a?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=releaseassetproduction%2F20250629%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20250629T162438Z&X-Amz-Expires=1800&X-Amz-Signature=b300b3ffb2d4fd039d3f2c0a3890a84c643b47f0dfd27431921b7f32a048e0b3&X-Amz-SignedHeaders=host&response-content-disposition=attachment%3B%20filename%3Dmihomo-linux-amd64-v1.19.11.gz&response-content-type=application%2Foctet-stream"
}

fn copy_service_file(name: impl AsRef<str>, template: &[u8]) -> color_eyre::Result<bool> {
    println!("正在安装服务: {}", name.as_ref());
    let name = name.as_ref();

    let systemd_dir = Path::new(SYSTEMD_DIR_STR);
    let service_file_name = format!("{}.service", name);
    let service_file_path = systemd_dir.join(&service_file_name);
    let mut service_file = if service_file_path.exists() {
        let over_write = Confirm::new(&format!("{} 已经存在，是否覆盖？", service_file_name))
            .with_default(false)
            .prompt()?;
        if !over_write {
            println!("跳过安装服务 {}", name);
            return Ok(false);
        }
        OpenOptions::new().write(true).truncate(true).open(&service_file_path)?
    } else {
        File::create_new(service_file_path)?
    };
    service_file.write_all(template)?;

    Ok(true)
}

fn load_service(name: impl AsRef<str>) -> color_eyre::Result<()> {
    let name = name.as_ref();
    println!("重载 systemd 配置...");
    systemctl_command(["daemon-reload"])?;
    println!("启用服务 {}...", name);
    systemctl_command(["enable", name])?;
    Ok(())
}

fn start_service(name: impl AsRef<str>) -> color_eyre::Result<()> {
    let name = name.as_ref();
    println!("启动服务 {}...", name);
    systemctl_command(["start", name])?;
    Ok(())
}

fn systemctl_command<I, T>(args: I) -> color_eyre::Result<()>
where
    T: AsRef<OsStr>,
    I: IntoIterator<Item = T>,
{
    let status = Command::new("systemctl")
        .args(args)
        .spawn()
        .wrap_err_with(|| "无法执行 systemctl 命令")?
        .wait()
        .wrap_err_with(|| "无法等待 systemctl 命令执行完成")?;

    if !status.success() {
        return Err(eyre!("systemctl 命令执行失败，状态码: {}", status));
    }

    Ok(())
}
