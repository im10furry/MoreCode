use std::path::PathBuf;

use mc_config::{AppConfig, ConfigLoader};
use mc_memory::MemorySystem;

use crate::cli::Cli;

pub struct AppContext {
    pub cwd: PathBuf,
    pub project_root: PathBuf,
    pub config: AppConfig,
    pub memory: MemorySystem,
}

impl AppContext {
    pub async fn initialize(cli: &Cli) -> Result<Self, String> {
        let project_root = match &cli.project_root {
            Some(path) => path.clone(),
            None => std::env::current_dir().map_err(|error| error.to_string())?,
        };
        let cwd = project_root.clone();

        let config_loader =
            ConfigLoader::with_default_paths_for(&project_root).map_err(|error| error.to_string())?;
        let config = config_loader
            .load()
            .await
            .map_err(|error| error.to_string())?;
        let memory = MemorySystem::new(&project_root)
            .await
            .map_err(|error| error.to_string())?;

        Ok(Self {
            cwd,
            project_root,
            config,
            memory,
        })
    }
}
