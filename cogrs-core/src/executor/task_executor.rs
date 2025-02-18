use crate::inventory::host::Host;
use crate::playbook::task::Task;
use log::debug;

pub struct TaskExecutor;

impl TaskExecutor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&self, host: &Host, task: &Task) {
        debug!(
            "executor run() - task {}, host: {}",
            task.uuid(),
            host.name()
        )
        // TODO: handle with_*
    }
}
