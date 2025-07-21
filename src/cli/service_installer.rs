use crate::api::UniversalProviderApi;
use crate::common::config::ConvertorConfig;
use crate::common::config::proxy_client::ProxyClient;
use crate::core::profile::Profile;
use crate::core::profile::clash_profile::ClashProfile;
use crate::core::renderer::Renderer;
use crate::core::renderer::clash_renderer::ClashRenderer;
use clap::ValueEnum;
use color_eyre::eyre::{WrapErr, eyre};
use flate2::bufread::GzDecoder;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::Confirm;
use reqwest::{Method, StatusCode};
use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio_stream::StreamExt;
use url::Url;

const SYSTEMD_DIR_STR: &str = "/etc/systemd/system";

const CONVERTOR_SERVICE_TEMPLATE: &[u8] = include_bytes!("../../assets/service/convertor.service");
const MIHOMO_SERVICE_TEMPLATE: &[u8] = include_bytes!("../../assets/service/mihomo.service");

#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum ServiceName {
    Convertor,
    Mihomo,
}

impl ServiceName {
    pub fn as_str(&self) -> &str {
        match self {
            ServiceName::Convertor => "convertor",
            ServiceName::Mihomo => "mihomo",
        }
    }
}

impl Display for ServiceName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub struct ServiceInstaller {
    pub name: ServiceName,
    pub base_dir: PathBuf,
    pub config: ConvertorConfig,
    pub api: UniversalProviderApi,
}

impl ServiceInstaller {
    pub fn new(name: ServiceName, base_dir: PathBuf, config: ConvertorConfig, api: UniversalProviderApi) -> Self {
        ServiceInstaller {
            name,
            base_dir,
            config,
            api,
        }
    }

    pub async fn install(self) -> color_eyre::Result<()> {
        if matches!(self.name, ServiceName::Mihomo) {
            self.download_mihomo().await?;
            self.copy_mihomo_executable()?;
            self.generate_mihomo_config().await?;
            self.install_mihomo_ui().await?;
        }

        let status = self.copy_service_file()?;
        if !status {
            println!("跳过拷贝 systemd 配置: {}", self.name);
            return Ok(());
        };

        self.load_service()?;

        self.start_service()?;

        println!("服务: {} 安装成功", self.name);
        Ok(())
    }

    async fn download_mihomo(&self) -> color_eyre::Result<()> {
        let cache_dir = self.base_dir.join("cache");
        let executable_path = cache_dir.join("mihomo");
        if executable_path.exists() {
            println!("mihomo 已经存在，跳过下载");
            return Ok(());
        }

        println!("获取最新版本信息...");
        let latest_url = "https://github.com/MetaCubeX/mihomo/releases/latest/download";
        let version_txt_url = Url::parse(&format!("{latest_url}/version.txt"))?;

        let http_response = self
            .api
            .client()
            .request(Method::GET, version_txt_url)
            .send()
            .await?
            .error_for_status()?;
        if http_response.status() != StatusCode::OK {
            return Err(eyre!("无法获取最新版本信息: {}", http_response.status()));
        }

        let version = http_response.text().await?;
        let version = version.trim();
        println!("最新版本: {version}");

        let compressed_name = format!("mihomo-linux-amd64-{version}.gz");
        let compressed_path = cache_dir.join(&compressed_name);
        let compressed = if !compressed_path.exists() {
            println!("正在下载 {} ...", &compressed_name);
            let download_url = format!("{latest_url}/{compressed_name}");
            let http_response = self
                .api
                .client()
                .request(Method::GET, &download_url)
                .send()
                .await?
                .error_for_status()?;
            if http_response.status() != StatusCode::OK {
                return Err(eyre!("无法下载 mihomo: {}", http_response.status()));
            }

            let mut downloaded_size = 0u64;
            let total_size = http_response.content_length().unwrap_or(0u64);
            if total_size == 0 {
                return Err(eyre!("下载的文件大小为 0，可能是版本信息错误"));
            }
            let mut stream = http_response.bytes_stream();
            let mut compressed = vec![];
            let progress_bar = ProgressBar::new(total_size).with_style(ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
            )?);
            while let Some(Ok(chunk)) = stream.next().await {
                downloaded_size += chunk.len() as u64;
                compressed.extend(chunk);
                progress_bar.set_position(downloaded_size);
            }
            progress_bar.set_position(downloaded_size);
            progress_bar.finish();
            if downloaded_size != total_size {
                return Err(eyre!(
                    "下载的文件大小不匹配，期望: {}, 实际: {}",
                    total_size,
                    downloaded_size
                ));
            }
            println!("下载完成，大小: {downloaded_size} bytes");
            println!("写入到 {}", compressed_path.display());
            tokio::fs::write(compressed_path, &compressed).await?;
            compressed
        } else {
            println!("已存在: {}", compressed_path.display());
            tokio::fs::read(compressed_path).await?
        };

        println!("开始解压缩 mihomo...");
        let mut decoder = GzDecoder::new(compressed.as_slice());
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        println!("解压缩完成，大小: {} bytes", decompressed.len());
        let mihomo_path = cache_dir.join("mihomo");
        println!("写入到 {}", mihomo_path.display());
        tokio::fs::write(&mihomo_path, &decompressed).await?;
        Ok(())
    }

    fn copy_mihomo_executable(&self) -> color_eyre::Result<()> {
        let executable_path = self.base_dir.join("cache").join("mihomo");
        println!("安装 mihomo 到 /usr/local/bin/mihomo");
        pkexec_command(["cp", "-f", executable_path.to_str().unwrap(), "/usr/local/bin/mihomo"])?;
        println!("设置可执行权限...");
        pkexec_command(["chmod", "+x", "/usr/local/bin/mihomo"])?;
        println!("mihomo 复制完成");
        Ok(())
    }

    async fn generate_mihomo_config(&self) -> color_eyre::Result<()> {
        #[cfg(debug_assertions)]
        let mihomo_path = self.base_dir.join("mihomo");
        #[cfg(not(debug_assertions))]
        let mihomo_path = std::env::home_dir()
            .ok_or(eyre!("无法获取用户主目录"))?
            .join(".config")
            .join("mihomo");

        if !mihomo_path.exists() {
            println!("正在创建 mihomo 配置目录: {}", mihomo_path.display());
            tokio::fs::create_dir_all(&mihomo_path)
                .await
                .wrap_err_with(|| format!("无法创建 mihomo 配置目录: {}", mihomo_path.display()))?;
        }

        let config_path = mihomo_path.join("config.yaml");
        if config_path.exists() {
            println!("mihomo 配置文件已存在，跳过生成: {}", config_path.display());
            return Ok(());
        }

        println!("正在生成 mihomo 配置文件: {}", config_path.display());
        let convertor_url = self.config.create_convertor_url(ProxyClient::Clash)?;
        let clash_profile_content = self.api.get_raw_profile(ProxyClient::Clash).await?;
        let profile = ClashProfile::parse(clash_profile_content)?;

        let mut template = ClashProfile::template()?;
        template.patch(profile)?;
        template.convert(&convertor_url)?;
        let config_content = ClashRenderer::render_profile(&template)?;
        tokio::fs::write(&config_path, &config_content).await?;
        println!("mihomo 配置文件生成成功: {}", config_path.display());
        Ok(())
    }

    async fn install_mihomo_ui(&self) -> color_eyre::Result<()> {
        #[cfg(debug_assertions)]
        let mihomo_path = self.base_dir.join("mihomo");
        #[cfg(not(debug_assertions))]
        let mihomo_path = std::env::home_dir()
            .ok_or(eyre!("无法获取用户主目录"))?
            .join(".config")
            .join("mihomo");

        println!("安装 mihomo web-ui...");
        let ui_path = mihomo_path.join("ui");
        if ui_path.exists() {
            return Ok(());
        }
        let status = Command::new("git")
            .current_dir(&mihomo_path)
            .args([
                "clone",
                "https://github.com/metacubex/metacubexd.git",
                "-b",
                "gh-pages",
                "ui",
            ])
            .spawn()?
            .wait()?;
        if !status.success() {
            return Err(eyre!("克隆 UI 仓库失败: {}", status));
        }
        println!("mihomo web-ui 安装完成");
        Ok(())
    }

    fn copy_service_file(&self) -> color_eyre::Result<bool> {
        println!("正在安装服务: {}", self.name);

        let systemd_dir = Path::new(SYSTEMD_DIR_STR);
        let service_file_name = format!("{}.service", self.name);
        let service_file_path = systemd_dir.join(&service_file_name);
        let mut service_file = if service_file_path.exists() {
            let over_write = Confirm::new(&format!("{service_file_name} 已经存在，是否覆盖？"))
                .with_default(false)
                .prompt()?;
            if !over_write {
                println!("跳过安装服务 {}", self.name);
                return Ok(false);
            }
            OpenOptions::new().write(true).truncate(true).open(&service_file_path)?
        } else {
            File::create_new(service_file_path)?
        };

        let template = match self.name {
            ServiceName::Convertor => CONVERTOR_SERVICE_TEMPLATE,
            ServiceName::Mihomo => MIHOMO_SERVICE_TEMPLATE,
        };

        service_file.write_all(template)?;

        Ok(true)
    }

    fn load_service(&self) -> color_eyre::Result<()> {
        println!("重载 systemd 配置...");
        systemctl_command(["daemon-reload"])?;
        println!("启用服务 {}...", self.name);
        systemctl_command(["enable", self.name.as_str()])?;
        Ok(())
    }

    fn start_service(&self) -> color_eyre::Result<()> {
        println!("启动服务 {}...", self.name);
        systemctl_command(["start", self.name.as_str()])?;
        Ok(())
    }
}

fn systemctl_command(args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> color_eyre::Result<()> {
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

fn pkexec_command(args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> color_eyre::Result<()> {
    let status = Command::new("pkexec")
        .args(args)
        .spawn()
        .wrap_err_with(|| "无法执行 pkexec 命令")?
        .wait()
        .wrap_err_with(|| "无法等待 pkexec 命令执行完成")?;

    if !status.success() {
        return Err(eyre!("pkexec 命令执行失败，状态码: {}", status));
    }

    Ok(())
}
