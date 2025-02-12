use crate::executor::play_iterator::PlayIterator;
use crate::executor::task_queue_manager::TaskQueueManager;

pub struct LinearStrategy<'a> {
    tqm: &'a TaskQueueManager<'a>,
}

impl<'a> LinearStrategy<'a> {
    pub fn new(tqm: &'a TaskQueueManager) -> Self {
        LinearStrategy { tqm }
    }

    pub fn run(&self, iterator: &PlayIterator) {}
}
