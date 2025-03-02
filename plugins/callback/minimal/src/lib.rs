use anyhow::Result;
use cogrs_plugins::callback::{CallbackPlugin, EventType};
use cogrs_plugins::create_callback_plugin;
use cogrs_plugins::plugin_type::PluginType;
use std::collections::HashMap;
use std::sync::Arc;

create_callback_plugin!(
    MinimalStdOut,
    "minimal",
    HashMap::from([("cogrs-plugin", "1.2.3"),]),
    [
        EventType::RunnerOnOk,
        EventType::RunnerOnFailed,
        EventType::RunnerOnSkipped,
        EventType::RunnerOnUnreachable,
        EventType::OnFileDiff
    ],
    |event: &EventType, _data| {
        match event {
            EventType::RunnerOnOk => {
                println!("RunnerOnOk");
            }
            EventType::RunnerOnFailed => {
                println!("RunnerOnFailed");
            }
            EventType::RunnerOnSkipped => {
                println!("RunnerOnSkipped");
            }
            EventType::RunnerOnUnreachable => {
                println!("RunnerOnUnreachable");
            }
            EventType::OnFileDiff => {
                println!("OnFileDiff");
            }
            _ => {
                println!("Unknown event");
            }
        }

        Ok(())
    }
);
