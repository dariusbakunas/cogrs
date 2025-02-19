use crate::executor::task_result::TaskResult;
use crate::executor::worker_message::WorkerMessage;
use crate::inventory::host::Host;
use crate::playbook::task::Task;
use anyhow::Result;
use log::debug;
use tokio::sync::mpsc;

pub struct TaskExecutor;

impl TaskExecutor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(
        &self,
        host: &Host,
        task: &Task,
        sender: &mpsc::Sender<WorkerMessage>,
    ) -> Result<TaskResult> {
        debug!(
            "executor run() - task {}, host: {}",
            task.uuid(),
            host.name()
        );
        let result = TaskResult::new(host.name(), task.uuid());
        // TODO: handle with_*
        // TODO: get connection plugin

        Ok(result)
    }
}
