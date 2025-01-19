mod cli;
mod inventory;

use crate::inventory::manager;
use anyhow::Result;

pub fn run(inventory: Option<&Vec<String>>) -> Result<()> {
    let mut manager = manager::InventoryManager::new();
    manager.parse_sources(inventory)?;
    Ok(())
}
