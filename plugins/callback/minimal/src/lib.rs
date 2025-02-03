use anyhow::Result;
use cogrs_plugins::callback::{CallbackPlugin, EventType};
use cogrs_plugins::create_callback_plugin;

create_callback_plugin!(
    MinimalStdOut,
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
