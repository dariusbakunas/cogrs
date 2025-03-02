use anyhow::Result;
use cogrs_modules::define_module;
use cogrs_modules::framework::Module;
use cogrs_schema::define_schema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize, Default)]
struct Parameters {
    cmd: String,
}

define_schema! {
    Parameters,
    r#"
    {
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "Command Module",
        "type": "object",
        "properties": {
            "cmd": { "type": "string", "description": "The command to run." }
        },
        "additionalProperties": false,
        "required": ["cmd"]
    }
    "#
}

pub struct CommandModule;

impl Module for CommandModule {
    fn schema() -> &'static str {
        SCHEMA
    }

    fn run(inputs: Value) -> Result<()> {
        let parameters: Parameters = serde_json::from_value(inputs)?;
        let cmd = parameters.cmd.clone();
        println!("Running command: {}", cmd);

        Ok(())
    }
}

define_module!(CommandModule);
