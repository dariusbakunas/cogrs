use crate::executor::worker_message::WorkerMessage;
use crate::inventory::host::Host;
use crate::playbook::task::Task;
use crate::vars::variable::Variable;
use anyhow::Result;
use cogrs_modules::action_handler::ActionHandler;
use cogrs_modules::task_result::TaskResult;
use cogrs_plugins::connection::ConnectionPlugin;
use cogrs_plugins::shell::ShellPlugin;
use indexmap::IndexMap;
use log::debug;
use tokio::sync::mpsc;

pub struct TaskExecutor;

#[cfg(feature = "static-plugins")]
async fn load_plugins(
    task: &Task,
    mut task_vars: IndexMap<String, Variable>,
) -> Result<(Box<dyn ConnectionPlugin>, Box<dyn ShellPlugin>)> {
    use sh_lib::Sh;
    use ssh_lib::Ssh;

    let mut connection_plugin = Box::new(Ssh::default());

    let parameters = serde_json::to_string(&task_vars)?;
    connection_plugin.initialize(&parameters)?;

    let shell_plugin = Box::new(Sh::default());
    Ok((connection_plugin, shell_plugin))
}

#[cfg(not(feature = "static-plugins"))]
async fn load_plugins(
    task: &Task,
    mut task_vars: IndexMap<String, Variable>,
) -> Result<(Box<dyn ConnectionPlugin>, Box<dyn ShellPlugin>)> {
    let plugin_loader = cogrs_plugins::plugin_loader::PluginLoader::instance();
    let mut loader = plugin_loader.lock().await;

    let shell_plugin = loader.get_shell_plugin("sh").await?;
    let mut connection_plugin = loader.get_connection_plugin(task.connection()).await?;
    let parameters = serde_json::to_string(&task_vars)?;
    connection_plugin.initialize(&parameters)?;
    Ok((connection_plugin, shell_plugin))
}

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

        task_vars.insert(
            String::from("remote_user"),
            Variable::String("test-user".to_string()),
        );

        // TODO: handle conditionals

        let result = TaskResult::new(host.name(), task.uuid());
        // TODO: handle with_*
        // TODO: get connection plugin
        let (connection_plugin, shell_plugin) = load_plugins(task, task_vars).await?;
        let action_handler = ActionHandler::new(connection_plugin, shell_plugin);
        action_handler.run().await?;

        Ok(result)
    }

    fn get_connection(current_connection: &str) {}
}
