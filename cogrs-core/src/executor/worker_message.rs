use cogrs_modules::task_result::TaskResult;
use cogrs_plugins::callback::EventType;

pub enum WorkerMessage {
    Callback((EventType, Option<TaskResult>)),
    Display(String),
    Prompt(String),
}
