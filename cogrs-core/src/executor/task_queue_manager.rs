use crate::executor::play_iterator::PlayIterator;
use crate::inventory::manager::InventoryManager;
use crate::playbook::play::Play;
use crate::strategy::linear::LinearStrategy;
use crate::strategy::Strategy;
use crate::vars::manager::VariableManager;
use anyhow::Result;
use cogrs_plugins::callback::{CallbackPlugin, EventType};
use serde_json::Value;
use std::cmp::min;
use std::collections::HashMap;
use std::sync::Arc;

pub struct TaskQueueManager<'a> {
    forks: u32,
    callbacks_loaded: bool,
    inventory_manager: &'a InventoryManager,
    variable_manager: &'a VariableManager,
    callbacks: HashMap<EventType, Vec<Arc<dyn CallbackPlugin>>>,
}

const DEFAULT_FORKS: u32 = 5;

impl<'a> TaskQueueManager<'a> {
    pub fn new(
        forks: Option<u32>,
        inventory_manager: &'a InventoryManager,
        variable_manager: &'a VariableManager,
    ) -> Self {
        Self {
            callbacks: HashMap::new(),
            callbacks_loaded: false,
            forks: forks.unwrap_or(DEFAULT_FORKS),
            inventory_manager,
            variable_manager,
        }
    }

    pub async fn run(&mut self, play: &Play) -> Result<()> {
        self.load_callbacks(
            "/Users/darius/Programming/cogrs/dist/minimal-apple_x86_64-apple-darwin",
        );
        let all_vars = self
            .variable_manager
            .get_vars(play, None, self.inventory_manager);

        self.emit_event(EventType::PlaybookOnPlayStart, None).await;

        let mut play_iterator = PlayIterator::new();
        play_iterator.init(play, self.inventory_manager)?;

        let forks = min(self.forks, play_iterator.get_batch_size());

        match play.get_strategy() {
            Strategy::Linear => {
                let strategy = LinearStrategy::new(&self);
                strategy.run(&play_iterator);
            }
            Strategy::Free => {
                todo!()
            }
        }

        Ok(())
    }

    pub fn register_callback(&mut self, callback: Box<dyn CallbackPlugin>) {
        let callback: Arc<dyn CallbackPlugin> = Arc::from(callback);
        for event in callback.get_interested_events() {
            self.callbacks
                .entry(event)
                .or_insert_with(Vec::new)
                .push(callback.clone());
        }
    }

    fn load_callbacks(&mut self, plugin_dir: &str) {
        use libloading::{Library, Symbol};
        use std::fs;

        let plugin_extension = if cfg!(target_os = "windows") {
            "dll"
        } else if cfg!(target_os = "macos") {
            "dylib"
        } else {
            "so"
        };

        if self.callbacks_loaded {
            return;
        }

        for entry in fs::read_dir(plugin_dir).expect("Invalid plugin directory") {
            let path = entry.expect("Failed to read entry").path();
            if path.extension().and_then(|e| e.to_str()) == Some(plugin_extension) {
                unsafe {
                    let lib = Library::new(&path).expect("Failed to load plugin");

                    // Dynamically load the callback creation function
                    let create_callback: Symbol<fn() -> Box<dyn CallbackPlugin>> = lib
                        .get(b"create_plugin")
                        .expect("Failed to find create_plugin function");

                    let plugin = create_callback();

                    // Register the plugin for events
                    self.register_callback(plugin);
                }
            }
        }

        self.callbacks_loaded = true
    }

    pub async fn emit_event(&self, event: EventType, data: Option<Value>) {
        if let Some(callbacks) = self.callbacks.get(&event) {
            // Spawn and collect tasks
            let tasks: Vec<_> = callbacks
                .iter()
                .map(|callback| {
                    let callback = callback.clone();
                    let event = event.clone();
                    let data = data.clone();
                    tokio::spawn(async move {
                        callback.on_event(&event, data.as_ref()); // Async invocation
                    })
                })
                .collect();

            // Wait for all spawned tasks to complete
            for task in tasks {
                if let Err(err) = task.await {
                    eprintln!("Callback task panicked: {:?}", err);
                }
            }
        }
    }
}
