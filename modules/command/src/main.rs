use anyhow::Result;
use clap::Parser;
use common::framework::Module;
use common::{define_module, define_schema};

define_schema! {
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

    fn run(_inputs: serde_json::value::Value) -> Result<()> {
        Ok(())
    }
}

define_module!(CommandModule);
