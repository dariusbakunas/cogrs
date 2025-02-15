use crate::playbook::task::Task;
use crate::utils::get_unique_id;

#[derive(Clone)]
pub enum BlockEntry {
    Task(Task),
    Block(Box<Block>),
}

#[derive(Clone)]
pub struct Block {
    block: Vec<BlockEntry>,
    rescue: Vec<BlockEntry>,
    always: Vec<BlockEntry>,
    run_once: bool,
    implicit: bool,
    uuid: String,
}

impl Block {
    pub fn new() -> Self {
        Block {
            uuid: get_unique_id(false),
            block: Vec::new(),
            rescue: Vec::new(),
            always: Vec::new(),
            run_once: false,
            implicit: false,
        }
    }

    pub fn set_block_entries(&mut self, entries: Vec<BlockEntry>) {
        self.block = entries;
    }

    pub fn has_rescue_entries(&self) -> bool {
        !self.rescue.is_empty()
    }

    pub fn has_always_entries(&self) -> bool {
        !self.always.is_empty()
    }

    pub fn has_any_entries(&self) -> bool {
        !self.block.is_empty() || !self.rescue.is_empty() || !self.always.is_empty()
    }

    pub fn has_block_entries(&self) -> bool {
        !self.block.is_empty()
    }

    pub fn get_block_entry(&self, index: usize) -> Option<&BlockEntry> {
        self.block.get(index)
    }

    pub fn add_to_block(&mut self, entry: BlockEntry) {
        self.block.push(entry);
    }

    pub fn add_to_rescue(&mut self, entry: BlockEntry) {
        self.rescue.push(entry);
    }

    pub fn add_to_always(&mut self, entry: BlockEntry) {
        self.always.push(entry);
    }

    pub fn set_is_implicit(&mut self, value: bool) {
        self.implicit = value;
    }
}
