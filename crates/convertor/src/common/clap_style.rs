use clap::builder::Styles;
use clap::builder::styling::RgbColor;

// Sonokai · Truecolor（24-bit，更贴近主题）
pub const SONOKAI: Styles = Styles::styled()
    .header(RgbColor(0xFD, 0x97, 0x1F).on_default().bold().underline())
    .usage(RgbColor(0xA6, 0xE2, 0x2E).on_default().bold())
    .literal(RgbColor(0xF9, 0x26, 0x72).on_default().bold())
    .placeholder(RgbColor(0x66, 0xD9, 0xEF).on_default().italic().dimmed())
    .error(RgbColor(0xFF, 0x55, 0x55).on_default().bold())
    .valid(RgbColor(0xA6, 0xE2, 0x2E).on_default().bold())
    .invalid(RgbColor(0xFF, 0x55, 0x55).on_default());

pub const SONOKAI_TC: Styles = Styles::styled()
    // “Usage: / Options:” 标题：yellow，粗体+下划线
    .header(RgbColor(0xE7, 0xC6, 0x64).on_default().bold().underline())
    // Usage 行：green，粗体
    .usage(RgbColor(0x9E, 0xD0, 0x72).on_default().bold())
    // 字面量（命令/旗标）：orange，粗体
    .literal(RgbColor(0xF3, 0x96, 0x60).on_default().bold())
    // 占位符（<ARG>）：blue，斜体+弱化
    .placeholder(RgbColor(0x76, 0xCC, 0xE0).on_default().italic().dimmed())
    // 错误：red，粗体
    .error(RgbColor(0xFC, 0x5D, 0x7C).on_default().bold())
    // 校验通过/失败
    .valid(RgbColor(0x9E, 0xD0, 0x72).on_default().bold())
    .invalid(RgbColor(0xFC, 0x5D, 0x7C).on_default());
