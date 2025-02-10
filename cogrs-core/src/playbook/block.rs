use crate::playbook::task::Task;

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
}

impl Block {
    pub fn new(run_once: bool) -> Self {
        Block {
            block: Vec::new(),
            rescue: Vec::new(),
            always: Vec::new(),
            run_once,
        }
    }

    pub fn set_block_entries(&mut self, entries: Vec<BlockEntry>) {
        self.block = entries;
    }

    pub fn has_entries(&self) -> bool {
        !self.block.is_empty() && !self.rescue.is_empty() && !self.always.is_empty()
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
}
