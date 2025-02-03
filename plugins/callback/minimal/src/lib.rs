use anyhow::Result;
use cogrs_plugins::callback::{CallbackPlugin, EventType};
use cogrs_plugins::create_callback_plugin;

create_callback_plugin!(
    MinimalStdOut,
    [EventType::PlaybookOnStart, EventType::PlaybookOnPlayStart],
    |event: &EventType, _data: Option<&serde_json::Value>| {
        match event {
            EventType::PlaybookOnStart => {
                println!("Playbook started");
            }
            EventType::PlaybookOnPlayStart => {
                println!("Play started");
            }
            _ => println!("Unknown event: {:?}", event),
        }

        Ok(())
    }
);
