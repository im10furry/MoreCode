use std::env;

use mc_config::AppConfig;

#[derive(Debug, Clone)]
pub struct AppContext {
    pub cwd: std::path::PathBuf,
    pub config: AppConfig,
}

pub fn initialize() -> Result<AppContext, String> {
    let cwd = env::current_dir()
        .map_err(|error| format!("failed to resolve current directory: {error}"))?;
    Ok(AppContext {
        cwd,
        config: AppConfig::default(),
    })
}
