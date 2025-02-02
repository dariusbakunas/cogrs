use crate::play::Play;

pub struct TaskQueueManager {
    forks: u32,
}

impl TaskQueueManager {
    pub fn new(forks: u32) -> Self {
        Self { forks }
    }

    pub fn run(&self, play: &Play) {
        todo!();
    }
}
