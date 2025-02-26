use crate::config::manager::ConfigManager;
use anyhow::{anyhow, Result};
use cogrs_plugins::plugin_loader;
use cogrs_plugins::plugin_type::PluginType;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tokio::sync::Mutex;

pub trait Cli {
    async fn init() -> Result<()> {
        let config_manager = ConfigManager::instance();
        config_manager.lock().await.init()?;
        let (cogrs_home, _) = config_manager
            .lock()
            .await
            .get_config_value::<PathBuf>("COGRS_HOME")?
            .ok_or_else(|| anyhow!("`COGRS_HOME` is not defined in the configuration"))?;

        set_plugin_paths(config_manager).await?;

        if !cogrs_home.exists() {
            fs::create_dir_all(&cogrs_home)?;
        }

        Ok(())
    }
}

async fn set_plugin_paths(config_manager: &Mutex<ConfigManager>) -> Result<()> {
    let plugin_loader = plugin_loader::PluginLoader::instance();
    let mut loader = plugin_loader.lock().await;

    let mut plugin_paths: HashMap<PluginType, Vec<PathBuf>> = HashMap::new();

    let callback_plugin_paths: Vec<PathBuf> =
        get_plugin_paths(config_manager, "DEFAULT_CALLBACK_PLUGIN_PATH").await?;
    let connection_plugin_paths: Vec<PathBuf> =
        get_plugin_paths(config_manager, "DEFAULT_CONNECTION_PLUGIN_PATH").await?;
    let shell_plugin_paths: Vec<PathBuf> =
        get_plugin_paths(config_manager, "DEFAULT_SHELL_PLUGIN_PATH").await?;

    plugin_paths.insert(PluginType::Callback, callback_plugin_paths);
    plugin_paths.insert(PluginType::Connection, connection_plugin_paths);
    plugin_paths.insert(PluginType::Shell, shell_plugin_paths);

    loader.init(plugin_paths).await?;

    Ok(())
}

async fn get_plugin_paths(
    config_manager: &Mutex<ConfigManager>,
    config_key: &str,
) -> Result<Vec<PathBuf>> {
    // Retrieve the configuration value for the provided key
    let (config_value, _) = config_manager
        .lock()
        .await
        .get_config_value::<PathBuf>(config_key)?
        .ok_or_else(|| anyhow!("`{}` is not defined in the configuration", config_key))?;

    // Split the paths and map them into a vector of PathBuf
    let paths: Vec<PathBuf> = config_value
        .to_string_lossy()
        .split(':')
        .map(|path| PathBuf::from(path))
        .collect();

    Ok(paths)
}
