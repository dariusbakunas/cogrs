use crate::inventory::host::Host;
use crate::play::Play;
use crate::playbook::Playbook;
use crate::task::{Action, Task};
use crate::task_queue_manager::TaskQueueManager;
use anyhow::Result;
use log::info;

pub struct AdHoc;

impl AdHoc {
    pub fn run(
        module_name: &str,
        module_args: &str,
        hosts: &[Host],
        forks: u32,
        poll_interval: Option<u64>,
        task_timeout: Option<u64>,
        async_val: Option<u64>,
        one_line: bool,
    ) -> Result<()> {
        info!(
            "Running adhoc module {} with args {}",
            module_name, module_args
        );

        let task = Task::new(
            module_name,
            &Action::Module(module_name.to_string(), module_args.to_string()),
            poll_interval,
            async_val,
        );
        let tasks = vec![task];

        let play = Play::builder("CogRS Ad-Hoc", &tasks)
            .use_become(false)
            .gather_facts(false)
            .build();

        let _playbook = Playbook::new(String::from("__adhoc_playbook__"), &[play.clone()]);

        let tqm = TaskQueueManager::new(forks);
        tqm.run(&play);

        Ok(())
    }
}
