use crate::playbook::task::Task;
use crate::utils::get_unique_id;

#[derive(Clone, Debug)]
pub enum BlockEntry {
    Task(Task),
    Block(Box<Block>),
}

#[derive(Clone, Debug)]
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

    pub fn block_entries(&self) -> &Vec<BlockEntry> {
        &self.block
    }

    pub fn always_entries(&self) -> &Vec<BlockEntry> {
        &self.always
    }

    pub fn rescue_entries(&self) -> &Vec<BlockEntry> {
        &self.rescue
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

    pub fn get_always_entry(&self, index: usize) -> Option<&BlockEntry> {
        self.always.get(index)
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

    fn evaluate_block(&self, block: &Block) -> Vec<Task> {
        let mut tasks: Vec<Task> = Vec::new();
        tasks.extend(self.evaluate_and_append_task(block.block_entries()));
        tasks.extend(self.evaluate_and_append_task(block.rescue_entries()));
        tasks.extend(self.evaluate_and_append_task(block.always_entries()));
        tasks
    }
    fn evaluate_and_append_task(&self, entries: &[BlockEntry]) -> Vec<Task> {
        let mut tasks: Vec<Task> = Vec::new();

        for entry in entries {
            match entry {
                BlockEntry::Task(task) => tasks.push(task.clone()),
                BlockEntry::Block(block) => tasks.append(&mut self.evaluate_block(block)),
            }
        }

        tasks
    }
    pub fn get_tasks(&self) -> Vec<Task> {
        self.evaluate_block(self)
    }
}
