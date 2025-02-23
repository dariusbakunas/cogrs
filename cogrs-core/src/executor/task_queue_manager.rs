use crate::executor::play_iterator::PlayIterator;
use crate::inventory::host::Host;
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
use std::path::PathBuf;
use std::sync::Arc;

pub struct TaskQueueManager {
    forks: usize,
    callbacks_loaded: bool,
    callbacks: HashMap<EventType, Vec<Arc<dyn CallbackPlugin>>>,
    terminated: bool,
    unreachable_hosts: HashMap<String, Host>,
    workers: Vec<tokio::task::JoinHandle<()>>,
    callback_plugin_paths: Vec<PathBuf>,
}

const DEFAULT_FORKS: usize = 5;

impl TaskQueueManager {
    pub fn new(forks: Option<usize>, callback_plugin_paths: &[PathBuf]) -> Self {
        Self {
            callbacks: HashMap::new(),
            callbacks_loaded: false,
            forks: forks.unwrap_or(DEFAULT_FORKS),
            terminated: false,
            unreachable_hosts: HashMap::new(),
            workers: Vec::with_capacity(forks.unwrap_or(DEFAULT_FORKS)),
            callback_plugin_paths: callback_plugin_paths.to_owned(),
        }
    }

    pub fn get_worker(&mut self, index: usize) -> Option<&tokio::task::JoinHandle<()>> {
        self.workers.get(index)
    }

    pub fn set_worker(&mut self, index: usize, worker: tokio::task::JoinHandle<()>) {
        self.workers.insert(index, worker);
    }

    /// Iterates over the roles/tasks in a play, using the given (or default)
    /// strategy for queueing tasks. The default is the linear strategy, which
    /// operates like classic Ansible by keeping all hosts in lock-step with
    /// a given task (meaning no hosts move on to the next task until all hosts
    /// are done with the current task).
    pub async fn run(
        &mut self,
        play: Play,
        variable_manager: &VariableManager,
        inventory_manager: &InventoryManager,
    ) -> Result<()> {
        self.load_callbacks().await?;
        let all_vars = variable_manager.get_vars(Some(&play), None, None, None, true, true);

        self.emit_event(EventType::PlaybookOnPlayStart, None).await;

        let strategy = *play.strategy();

        let mut play_iterator = PlayIterator::new(play);
        play_iterator.init(inventory_manager)?;

        self.forks = min(self.forks, play_iterator.batch_size());

        match strategy {
            Strategy::Linear => {
                let mut strategy = LinearStrategy::new(self, inventory_manager, variable_manager);
                strategy.run(&mut play_iterator).await?;
            }
            Strategy::Free => {
                todo!()
            }
        }

        Ok(())
    }

    pub fn register_callback(&mut self, callback: Arc<dyn CallbackPlugin>) {
        for event in callback.get_interested_events() {
            self.callbacks
                .entry(event)
                .or_insert_with(Vec::new)
                .push(callback.clone());
        }
    }

    pub fn get_unreachable_hosts(&self) -> &HashMap<String, Host> {
        &self.unreachable_hosts
    }

    pub fn is_terminated(&self) -> bool {
        self.terminated
    }

    async fn load_callbacks(&mut self) -> Result<()> {
        let plugin_loader = cogrs_plugins::plugin_loader::PluginLoader::instance();
        let loader = plugin_loader.lock().await;

        let plugins = loader
            .get_callback_plugins(&self.callback_plugin_paths)
            .await?;

        for plugin in plugins {
            self.register_callback(plugin);
        }

        self.callbacks_loaded = true;
        Ok(())
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

    pub fn forks(&self) -> usize {
        self.forks
    }
}
