mod inventory;
mod cli;

use anyhow::Result;
use crate::inventory::manager;

pub fn run(inventory: Option<&Vec<String>>) -> Result<()> {
    let mut manager = manager::InventoryManager::new();
    manager.parse_sources(inventory)?;
    Ok(())
}