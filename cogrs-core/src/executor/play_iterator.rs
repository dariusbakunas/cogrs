use crate::executor::host_state::HostState;
use crate::inventory::manager::InventoryManager;
use crate::playbook::block::{Block, BlockEntry};
use crate::playbook::play::Play;
use crate::playbook::task::{Action, Task, TaskBuilder};
use anyhow::Result;
use log::info;
use std::collections::HashMap;

pub struct PlayIterator {
    blocks: Vec<BlockEntry>,
    batch_size: u32,
    host_states: HashMap<String, HostState>,
    end_play: bool,
    cur_task: u32,
}

impl PlayIterator {
    pub fn new() -> Self {
        PlayIterator {
            blocks: vec![],
            batch_size: 0,
            host_states: HashMap::new(),
            end_play: false,
            cur_task: 0,
        }
    }

    pub fn init(&mut self, play: &Play, inventory_manager: &InventoryManager) -> Result<()> {
        let mut setup_block = Block::new();
        let batch = inventory_manager.filter_hosts(play.get_pattern(), None)?;
        self.batch_size = batch.len() as u32;

        let mut setup_task_builder =
            TaskBuilder::new(Action::Module("setup".to_string(), "".to_string()));

        // Unless play is specifically tagged, gathering should 'always' run
        if play.get_tags().is_empty() {
            setup_task_builder = setup_task_builder.tags(vec!["always".to_string()]);
        }

        let setup_task = setup_task_builder.build();

        setup_block.add_to_block(BlockEntry::Task(setup_task));
        self.blocks.push(BlockEntry::Block(Box::new(setup_block)));

        for block in play.compile() {
            // TODO: filter tagged tasks
            if let BlockEntry::Block(block) = block {
                if block.has_entries() {
                    self.blocks.push(BlockEntry::Block(block));
                }
            } else if let BlockEntry::Task(task) = block {
                info!("Adding task to play iterator");
            }
        }

        for host in batch {
            let host_state = HostState::new(&self.blocks);
            self.host_states.insert(host.name.to_string(), host_state);

            // TODO: handle start_at_task option here
        }

        Ok(())
    }

    pub fn get_batch_size(&self) -> u32 {
        self.batch_size
    }
}
