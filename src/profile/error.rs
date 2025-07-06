use thiserror::Error;

/// 所有解析失败场景的统一错误
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("规则解析失败 (第 {line} 行): {reason}")]
    Rule { line: usize, reason: String },

    #[error("规则类型解析失败 (第 {line} 行): {reason}")]
    RuleType { line: usize, reason: String },

    #[error("代理解析失败 (第 {line} 行): {reason}")]
    Proxy { line: usize, reason: String },

    #[error("代理组解析失败 (第 {line} 行): {reason}")]
    ProxyGroup { line: usize, reason: String },

    #[error("缺少必要配置段: {0}")]
    SectionMissing(&'static str),

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    FmtError(#[from] std::fmt::Error),

    #[error(transparent)]
    YamlError(#[from] serde_yaml::Error),
}

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("渲染失败: {0}")]
    Render(String),

    #[error(transparent)]
    FmtError(#[from] std::fmt::Error),
}
