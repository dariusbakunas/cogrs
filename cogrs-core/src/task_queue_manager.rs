use crate::inventory::manager::InventoryManager;
use crate::play::Play;
use crate::vars::VariableManager;
use cogrs_callbacks::{CallbackPlugin, EventType};
use std::collections::HashMap;

pub struct TaskQueueManager {
    forks: u32,
    callbacks_loaded: bool,
    inventory_manager: InventoryManager,
    variable_manager: VariableManager,
    callbacks: HashMap<EventType, Vec<Box<dyn CallbackPlugin>>>,
}

impl TaskQueueManager {
    pub fn new(
        forks: u32,
        inventory_manager: InventoryManager,
        variable_manager: VariableManager,
    ) -> Self {
        Self {
            callbacks: HashMap::new(),
            callbacks_loaded: false,
            forks,
            inventory_manager,
            variable_manager,
        }
    }

    pub fn run(&mut self, _play: &Play) {
        self.load_callbacks()

        // TODO: implement callback functionality
    }

    pub fn load_callbacks(&mut self) {
        if self.callbacks_loaded {
            return;
        }

        // TODO
        self.callbacks_loaded = true
    }
}
