use crate::executor::task_queue_manager::TaskQueueManager;
use crate::inventory::manager::InventoryManager;
use crate::playbook::play::Play;
use crate::playbook::task::{Action, Task};
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
}

impl AdHoc {
    pub async fn run(
        module_name: &str,
        module_args: &str,
        inventory_manager: &InventoryManager,
        options: &AdHocOptions,
    ) -> Result<()> {
        info!(
            "Running adhoc module {} with args {}",
            module_name, module_args
        );

        let task = Task::new(
            module_name,
            &Action::Module(module_name.to_string(), module_args.to_string()),
            None,
            options.poll_interval,
            options.async_val,
        );
        let tasks = vec![task];
        let roles = [];

        let variable_manager = VariableManager::new();

        let play = Play::builder("CogRS Ad-Hoc", &tasks, &roles)
            .use_become(false)
            .gather_facts(false)
            .build();

        let _playbook = Playbook::new(String::from("__adhoc_playbook__"), &[play.clone()]);

        let mut tqm = TaskQueueManager::new(options.forks, inventory_manager, &variable_manager);
        tqm.run(&play).await?;

        Ok(())
    }
}
