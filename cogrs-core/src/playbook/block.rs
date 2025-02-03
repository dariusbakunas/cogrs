use crate::playbook::task::Task;

pub struct Block {
    block: Vec<Task>,
    rescue: Vec<Task>,
    always: Vec<Task>,
}

impl Block {
    pub fn new() -> Self {
        Block {
            block: Vec::new(),
            rescue: Vec::new(),
            always: Vec::new(),
        }
    }
}
