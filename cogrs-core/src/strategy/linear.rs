use crate::executor::host_state::HostState;
use crate::executor::play_iterator::PlayIterator;
use crate::executor::task_queue_manager::TaskQueueManager;
use crate::inventory::host::Host;
use crate::inventory::manager::InventoryManager;
use crate::playbook::play::Play;
use crate::playbook::task::Task;
use anyhow::Result;
use std::collections::{HashMap, HashSet};

/// The linear strategy is simple - get the next task and queue
///         it for all hosts, then wait for the queue to drain before
///         moving on to the next task
pub struct LinearStrategy<'a> {
    tqm: &'a TaskQueueManager<'a>,
    inventory_manager: &'a InventoryManager,
    host_cache: Vec<String>,
}

impl<'a> LinearStrategy<'a> {
    pub fn new(tqm: &'a TaskQueueManager) -> Self {
        LinearStrategy {
            tqm,
            inventory_manager: tqm.get_inventory_manager(),
            host_cache: vec![],
        }
    }

    fn set_host_cache(&mut self, play: &Play, refresh: bool) -> Result<()> {
        if !refresh && !self.host_cache.is_empty() {
            return Ok(());
        }

        // TODO: check ansible logic here
        let pattern = play.get_pattern();
        let limit = play.get_limit();

        self.host_cache = self
            .inventory_manager
            .filter_hosts(pattern, limit)?
            .iter()
            .map(|h| h.get_name().to_string())
            .collect();

        Ok(())
    }

    fn get_hosts_left(&self) -> Vec<&Host> {
        self.host_cache
            .iter()
            .filter(|h| !self.tqm.get_unreachable_hosts().contains_key(*h))
            // we're assuming inventory should be able to return all hosts here
            .filter_map(|h| self.inventory_manager.get_host(h))
            .collect()
    }

    /// Returns a list of (host, task) tuples, where the task may
    ///         be a noop task to keep the iterator in lock step across
    ///         all hosts.
    fn get_next_task_lockstep(
        &self,
        hosts: Vec<&Host>,
        iterator: &mut PlayIterator,
    ) -> Result<Vec<(Host, Task)>> {
        let mut result: Vec<(Host, Task)> = vec![];
        let mut state_task_per_host: HashMap<String, (HostState, Task)> = HashMap::new();

        for host in hosts {
            let (state, task) = iterator.get_next_task_for_host(host, true)?;

            if let Some(task) = task {
                state_task_per_host.insert(host.get_name().to_string(), (state, task));
            }
        }

        if state_task_per_host.is_empty() {
            return Ok(result);
        }

        let task_uuids: HashSet<&str> = state_task_per_host
            .values()
            .map(|(_, task)| task.get_uuid())
            .collect();

        // TODO: finish this method

        Ok(result)
    }

    pub fn run(&mut self, iterator: &mut PlayIterator) -> Result<()> {
        self.set_host_cache(iterator.get_play(), false)?;

        while !self.tqm.is_terminated() {
            let hosts_left = self.get_hosts_left();

            let host_tasks = self.get_next_task_lockstep(hosts_left.clone(), iterator);

            if hosts_left.is_empty() {
                break;
            }
        }

        Ok(())
    }

    pub fn cleanup(&self) {}
}
