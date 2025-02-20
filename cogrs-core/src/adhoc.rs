use crate::executor::task_queue_manager::TaskQueueManager;
use crate::inventory::manager::InventoryManager;
use crate::playbook::play::Play;
use crate::playbook::play_context::PlayContextBuilder;
use crate::playbook::task::{Action, TaskBuilder};
use crate::playbook::Playbook;
use crate::vars::manager::VariableManager;
use anyhow::Result;
use log::info;
use std::path::PathBuf;

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

        let task = TaskBuilder::new(
            "AdHoc",
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

        let play_context = PlayContextBuilder::new()
            .connection_timeout(options.connection_timeout)
            .private_key_file(options.private_key_file.as_ref())
            .build();

        let mut tqm = TaskQueueManager::new(Some(options.forks as usize), &play_context);
        tqm.run(play, &variable_manager, inventory_manager).await?;

        Ok(())
    }
}
