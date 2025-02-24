use crate::executor::task_result::TaskResult;
use crate::executor::worker_message::WorkerMessage;
use crate::inventory::host::Host;
use crate::playbook::task::Task;
use crate::vars::variable::Variable;
use anyhow::Result;
use indexmap::IndexMap;
use log::debug;
use std::collections::HashMap;
use tokio::sync::mpsc;

pub struct TaskExecutor;

impl TaskExecutor {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run(
        &self,
        host: &Host,
        task: &Task,
        mut task_vars: IndexMap<String, Variable>,
        sender: &mpsc::Sender<WorkerMessage>,
    ) -> Result<TaskResult> {
        debug!(
            "executor run() - task {}, host: {}",
            task.uuid(),
            host.name()
        );

        task_vars.insert(
            String::from("host"),
            Variable::String(host.address().to_string()),
        );

        // TODO: handle conditionals

        let result = TaskResult::new(host.name(), task.uuid());
        // TODO: handle with_*
        // TODO: get connection plugin
        let plugin_loader = cogrs_plugins::plugin_loader::PluginLoader::instance();
        let mut loader = plugin_loader.lock().await;

        let mut connection_plugin = loader.get_connection_plugin(task.connection()).await?;
        let parameters = serde_json::to_string(&task_vars)?;
        //connection_plugin.initialize(&parameters)?;

        Ok(result)
    }

    fn get_connection(current_connection: &str) {}
}
