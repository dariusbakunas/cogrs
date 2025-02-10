use crate::executor::host_state::HostState;
use crate::inventory::manager::InventoryManager;
use crate::playbook::block::{Block, BlockEntry};
use crate::playbook::play::Play;
use crate::playbook::task::{Action, Task};
use anyhow::Result;
use std::collections::HashMap;

pub struct PlayIterator {
    blocks: Vec<Block>,
    batch_size: u32,
    host_states: HashMap<String, HostState>,
}

impl PlayIterator {
    pub fn new() -> Self {
        PlayIterator {
            blocks: vec![],
            batch_size: 0,
            host_states: HashMap::new(),
        }
    }

    pub fn init(&mut self, play: &Play, inventory_manager: &InventoryManager) -> Result<()> {
        let mut setup_block = Block::new(false);
        let batch = inventory_manager.filter_hosts(play.get_pattern(), None)?;
        self.batch_size = batch.len() as u32;

        let setup_task = Task::new(
            "Gathering Facts",
            &Action::Module("gather_facts".to_string(), "".to_string()),
            None,
            None,
            None,
            // Unless play is specifically tagged, gathering should 'always' run
            if play.get_tags().is_empty() {
                vec!["always".to_string()]
            } else {
                vec![]
            },
        );

        setup_block.add_to_block(BlockEntry::Task(setup_task));
        self.blocks.push(setup_block);

        for block in play.compile() {
            // TODO: filter tagged tasks
            if block.has_entries() {
                self.blocks.push(block);
            }
        }

        Ok(())
    }
}
