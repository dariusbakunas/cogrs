mod cli;
pub mod inventory;

use crate::inventory::manager;
use anyhow::Result;

pub fn run(inventory: Option<&Vec<String>>) -> Result<()> {
    let mut manager = manager::InventoryManager::new();
    manager.parse_sources(inventory)?;
    let hosts = manager.list_hosts();
    println!("{hosts:#?}");

    let groups = manager.list_groups();
    println!("{groups:#?}");
    Ok(())
}
