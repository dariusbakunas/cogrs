use crate::config::manager::ConfigManager;
use crate::executor::task_queue_manager::TaskQueueManager;
use crate::inventory::manager::InventoryManager;
use crate::playbook::play::Play;
use crate::playbook::task::{Action, TaskBuilder};
use crate::playbook::Playbook;
use crate::vars::manager::VariableManager;
use crate::vars::variable::Variable::Path;
use anyhow::{anyhow, Result};
use cogrs_plugins::plugin_type::PluginType;
use cogrs_plugins::{plugin_loader, plugin_type};
use log::info;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tokio::sync::Mutex;

pub struct AdHoc;

pub struct AdHocOptions {
    pub forks: u32,
    pub poll_interval: Option<u64>,
    pub task_timeout: Option<u64>,
    pub async_val: Option<u64>,
    pub one_line: bool,
    pub connection: String,
    pub connection_timeout: Option<u64>,
    pub private_key_file: Option<PathBuf>,
}

impl AdHoc {
    pub async fn run(
        pattern: &str,
        limit: Option<&str>,
        module_name: &str,
        module_args: Option<String>,
        inventory_manager: &InventoryManager,
        options: &AdHocOptions,
    ) -> Result<()> {
        info!(
            "Running adhoc module {} with args {:?}",
            module_name, module_args
        );

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

        let task = TaskBuilder::new(
            "AdHoc",
            &options.connection,
            Action::Module(module_name.to_string(), module_args),
        )
        .poll_interval(options.poll_interval)
        .async_val(options.async_val)
        .build();

        let tasks = vec![task];
        let roles = [];

        let variable_manager = VariableManager::new(inventory_manager.get_base_dir());

        let play = Play::builder("CogRS Ad-Hoc", &roles)
            .use_become(false)
            .gather_facts(false)
            .connection(&options.connection)
            .pattern(pattern)
            .limit(limit)
            .tasks(&tasks)
            .build();

        let _playbook = Playbook::new("__adhoc_playbook__", &[play.clone()]);

        let mut tqm = TaskQueueManager::new(Some(options.forks as usize));
        tqm.run(play, &variable_manager, inventory_manager).await?;

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

    plugin_paths.insert(PluginType::Callback, callback_plugin_paths);
    plugin_paths.insert(PluginType::Connection, connection_plugin_paths);
    loader.set_plugin_paths(plugin_paths);

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
