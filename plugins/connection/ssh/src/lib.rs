use anyhow::Result;
use cogrs_plugins::connection::ConnectionPlugin;
use cogrs_plugins::create_connection_plugin;
use cogrs_plugins::plugin_type::PluginType;
use cogrs_schema::define_schema;

define_schema! {
    r#"
    {
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "SSH Connection Plugin",
        "type": "object",
        "properties": {
            "host": { "type": "string", "description": "Hostname/IP to connect to." }
        },
        "additionalProperties": true,
        "required": ["host"]
    }
    "#
}

create_connection_plugin!(Ssh, "ssh");

impl Ssh {}

impl ConnectionPlugin for Ssh {
    fn connected(&self) -> bool {
        todo!()
    }

    fn get_remote_architecture(&self) -> Result<String> {
        todo!()
    }

    fn exec_command(&self, command: &str) -> Result<String> {
        todo!()
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
        todo!()
    }
}
