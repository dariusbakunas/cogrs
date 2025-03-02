use crate::task_result::TaskResult;
use anyhow::Result;
use cogrs_plugins::connection::{CommandOutput, ConnectionPlugin};
use cogrs_plugins::shell::ShellPlugin;
use std::path::Path;

pub struct ActionHandler {
    connection: Box<dyn ConnectionPlugin>,
    shell: Box<dyn ShellPlugin>,
}

impl ActionHandler {
    pub fn new(connection: Box<dyn ConnectionPlugin>, shell: Box<dyn ShellPlugin>) -> Self {
        ActionHandler { connection, shell }
    }

    pub async fn run(&self) -> Result<TaskResult> {
        println!("Running action");
        self.make_tmp_dir().await?;

        todo!()
    }

    async fn make_tmp_dir(&self) -> Result<()> {
        let tmp_dir = self.remote_expand_user("~/.cogrs/tmp").await?;
        Ok(())
    }

    async fn remote_expand_user(&self, path: &str) -> Result<String> {
        if !path.starts_with("~") {
            return Ok(path.to_owned());
        }

        let split_path = Path::new(path);

        let components: Vec<&str> = split_path
            .components()
            .map(|comp| comp.as_os_str().to_str().unwrap_or(""))
            .collect();

        let mut expanded_path = components[0].to_string();

        if expanded_path.eq("~") {
            let become_user = self.connection.become_user();

            if self.connection.do_become() && become_user.is_some() {
                expanded_path = format!("~{}", become_user.unwrap());
            } else {
                expanded_path = format!("~{}", self.connection.remote_user());
            }
        }

        let cmd = self.shell.expand_user(&expanded_path, None)?;
        let output = self.low_level_execute_command(&cmd, false, None).await?;

        Ok(expanded_path)
    }

    async fn low_level_execute_command(
        &self,
        command: &str,
        sudoable: bool,
        chdir: Option<String>,
    ) -> Result<CommandOutput> {
        let mut cmd = command;
        if let Some(chdir) = chdir {
            todo!()
        }

        let output = self.connection.exec_command(cmd).await?;
        Ok(output)
    }

    fn make_tmp_path(&self) -> Result<()> {
        Ok(())
    }
}
