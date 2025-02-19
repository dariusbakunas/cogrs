use crate::inventory::host::Host;
use crate::playbook::task::Task;

pub struct TaskResult {
    host_name: String,
    task_uuid: String,
    failed: bool,
    changed: bool,
    skipped: bool,
    unreachable: bool,
    failed_when_result: bool,
    attempts: u32,
    retries: u32,
}

impl TaskResult {
    pub fn new(host_name: &str, task_uuid: &str) -> Self {
        TaskResult {
            host_name: host_name.to_string(),
            task_uuid: task_uuid.to_string(),
            failed: false,
            changed: false,
            skipped: false,
            unreachable: false,
            failed_when_result: false,
            attempts: 0,
            retries: 0,
        }
    }
}
