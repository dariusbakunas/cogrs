pub mod cli;
pub mod constants;
pub mod inventory;
pub mod vault;

use crate::inventory::manager;
use anyhow::Result;
use clap::Parser;
use cli::Cli;

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let inventory = cli.inventory.as_ref();

    let mut manager = manager::InventoryManager::new();
    manager.parse_sources(inventory)?;

    if cli.list_hosts {
        let mut hosts: Vec<String> = manager
            .filter_hosts(cli.limit.as_deref(), cli.pattern.as_str())?
            .iter()
            .map(|h| h.name.to_string())
            .collect();
        hosts.sort();
        for host in hosts {
            println!("{host}");
        }
        Ok(())
    } else {
        todo!();
    }
}
