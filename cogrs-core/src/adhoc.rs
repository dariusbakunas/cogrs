use crate::cli::Cli;
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

impl Cli for AdHoc {}

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

        Self::init().await?;

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
