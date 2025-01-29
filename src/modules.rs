use anyhow::Result;
use log::info;
use std::str::FromStr;

pub struct Modules {}

#[derive(Debug, Clone, Copy)]
pub enum ModuleType {
    Command,
    Shell,
}

impl FromStr for ModuleType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "command" => Ok(ModuleType::Command),
            "shell" => Ok(ModuleType::Shell),
            _ => Err(anyhow::format_err!("Module {} not supported", s)),
        }
    }
}

impl Default for Modules {
    fn default() -> Self {
        Self::new()
    }
}

impl Modules {
    pub fn new() -> Self {
        Modules {}
    }

    pub fn run(&self, module: ModuleType, _args: Option<&str>) {
        match module {
            ModuleType::Command => {
                info!("Running command");
            }
            ModuleType::Shell => {
                todo!();
            }
        }
    }
}
