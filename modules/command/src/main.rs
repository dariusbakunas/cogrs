use anyhow::Result;
use clap::Parser;
use cogrs_modules::define_module;
use cogrs_modules::framework::Module;
use cogrs_schema::define_schema;

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

    fn run(inputs: serde_json::value::Value) -> Result<()> {
        let cmd = inputs["cmd"].as_str().unwrap();
        println!("Running command: {}", cmd);

        Ok(())
    }
}

define_module!(CommandModule);
