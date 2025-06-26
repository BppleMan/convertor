use color_eyre::eyre::{eyre, WrapErr};
use inquire::Confirm;
use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process::Command;

const SYSTEMD_DIR_STR: &str = "/etc/systemd/system";

const CONVERTOR_SERVICE_TEMPLATE: &[u8] =
    include_bytes!("../assets/service/convertor.service");
const MIHOMO_SERVICE_TEMPLATE: &[u8] =
    include_bytes!("../assets/service/mihomo.service");

pub fn install_service(name: impl AsRef<str>) -> color_eyre::Result<()> {
    let name = name.as_ref();

    let template = match name {
        "convertor" => CONVERTOR_SERVICE_TEMPLATE,
        "mihomo" => MIHOMO_SERVICE_TEMPLATE,
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

fn copy_service_file(
    name: impl AsRef<str>,
    template: &[u8],
) -> color_eyre::Result<bool> {
    let name = name.as_ref();

    let systemd_dir = Path::new(SYSTEMD_DIR_STR);
    let service_file_name = format!("{}.service", name);
    let service_file_path = systemd_dir.join(&service_file_name);
    let mut service_file = if service_file_path.exists() {
        let over_write = Confirm::new(&format!(
            "{} 已经存在，是否覆盖？",
            service_file_name
        ))
        .with_default(false)
        .prompt()?;
        if !over_write {
            println!("跳过安装服务 {}", name);
            return Ok(false);
        }
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&service_file_path)?
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
