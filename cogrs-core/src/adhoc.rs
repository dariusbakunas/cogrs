use crate::executor::task_queue_manager::TaskQueueManager;
use crate::inventory::manager::InventoryManager;
use crate::playbook::play::Play;
use crate::playbook::task::{Action, TaskBuilder};
use crate::playbook::Playbook;
use crate::vars::manager::VariableManager;
use anyhow::Result;
use log::info;

pub struct AdHoc;

pub struct AdHocOptions {
    pub forks: u32,
    pub poll_interval: Option<u64>,
    pub task_timeout: Option<u64>,
    pub async_val: Option<u64>,
    pub one_line: bool,
    pub connection: String,
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

        let mut tqm = TaskQueueManager::new(Some(options.forks as usize));
        tqm.run(play, &variable_manager, inventory_manager).await?;

        Ok(())
    }
}
