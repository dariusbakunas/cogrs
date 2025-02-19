use crate::inventory::host::Host;
use crate::playbook::task::Task;

pub struct TaskResult {
    host_name: String,
    task_uuid: String,
}

impl TaskResult {
    pub fn new(host_name: &str, task_uuid: &str) -> Self {
        TaskResult {
            host_name: host_name.to_string(),
            task_uuid: task_uuid.to_string(),
        }
    }
}
