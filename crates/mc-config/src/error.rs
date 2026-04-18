use mc_core::McError;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("配置加载失败: {path}: {reason}")]
    LoadFailed { path: String, reason: String },
    #[error("配置解析失败: {path}: {reason}")]
    ParseFailed { path: String, reason: String },
    #[error("配置验证失败: {field}: {reason}")]
    ValidationFailed { field: String, reason: String },
    #[error("配置热重载失败: {reason}")]
    HotReloadFailed { reason: String },
    #[error("环境变量解析失败: {var_name}: {reason}")]
    EnvVarParseFailed { var_name: String, reason: String },
}

impl From<ConfigError> for McError {
    fn from(value: ConfigError) -> Self {
        match value {
            ConfigError::LoadFailed { path, reason } => McError::ConfigLoadFailed { path, reason },
            ConfigError::ParseFailed { path, reason } => {
                McError::ConfigParseFailed { path, reason }
            }
            ConfigError::ValidationFailed { field, reason } => {
                McError::ConfigValidationFailed { field, reason }
            }
            ConfigError::HotReloadFailed { reason } => McError::InternalError { message: reason },
            ConfigError::EnvVarParseFailed { var_name, reason } => McError::ConfigParseFailed {
                path: format!("env:{var_name}"),
                reason,
            },
        }
    }
}
