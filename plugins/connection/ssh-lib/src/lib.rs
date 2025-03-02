use anyhow::{Context, Result};
use async_trait::async_trait;
use cogrs_plugins::connection::{CommandOutput, ConnectionPlugin};
use cogrs_plugins::create_connection_plugin;
use cogrs_schema::define_schema;
use openssh::{KnownHosts, Session};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default)]
struct Parameters {
    host: String,
    task_uuid: String,
    #[serde(default, rename(deserialize = "become"))]
    do_become: bool,
    become_user: Option<String>,
    remote_user: String,
}

define_schema! {
    Parameters,
    r#"
    {
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "SSH Connection Plugin",
        "type": "object",
        "properties": {
            "host": { "type": "string", "description": "Hostname/IP to connect to." },
            "task_uuid": { "type": "string", "description": "Task UUID." },
            "become": { "type": "boolean", "description": "Whether to use sudo." },
            "become_user": { "type": "string", "description": "User to become." },
            "remote_user": { "type": "string", "description": "User to connect as." }
        },
        "additionalProperties": true,
        "required": ["host", "task_uuid", "remote_user"]
    }
    "#
}

create_connection_plugin!(Ssh, {
    parameters: Parameters,
});

impl Ssh {}

#[async_trait]
impl ConnectionPlugin for Ssh {
    fn do_become(&self) -> bool {
        self.parameters.do_become
    }

    fn become_user(&self) -> Option<String> {
        self.parameters.become_user.to_owned()
    }

    fn connected(&self) -> bool {
        todo!()
    }

    fn get_remote_architecture(&self) -> Result<String> {
        todo!()
    }

    async fn exec_command(&self, command: &str) -> Result<CommandOutput> {
        let host = self.parameters.host.to_owned();
        let task_uuid = self.parameters.task_uuid.to_owned();
        let become_user = self.become_user();
        let do_become = self.do_become();
        let remote_user = self.parameters.remote_user.to_owned();

        let connect_string = format!("{}@{}", remote_user, host);
        let session = Session::connect(&connect_string, KnownHosts::Accept)
            .await
            .with_context(|| format!("Failed to connect to {}.", host))?;

        let output = session.command(command).output().await?;
        session.close().await?;

        Ok(CommandOutput::new(
            String::from_utf8(output.stdout)?,
            String::from_utf8(output.stderr)?,
            output.status.code().unwrap_or(-1),
        ))
    }

    fn put_file(&self, source_path: &str, dest_path: &str) -> Result<()> {
        todo!()
    }

    fn fetch_file(&self, source_path: &str, dest_path: &str) -> Result<()> {
        todo!()
    }

    fn close(&self) {
        todo!()
    }

    fn connect(&self) -> Result<()> {
        todo!()
    }

    fn initialize(&mut self, parameters: &str) -> Result<()> {
        self.validate_parameters(parameters)?;
        self.parameters = serde_json::from_str(parameters)?;
        Ok(())
    }

    fn schema(&self) -> &'static str {
        SCHEMA
    }

    fn remote_user(&self) -> String {
        self.parameters.remote_user.to_owned()
    }
}
