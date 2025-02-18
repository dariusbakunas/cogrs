use crate::executor::host_state::HostState;
use crate::executor::play_iterator::PlayIterator;
use crate::executor::task_queue_manager::TaskQueueManager;
use crate::inventory::host::Host;
use crate::inventory::manager::InventoryManager;
use crate::playbook::block::BlockEntry;
use crate::playbook::play::Play;
use crate::playbook::task::Task;
use crate::vars::manager::VariableManager;
use anyhow::{anyhow, bail, Result};
use log::{debug, warn};
use std::collections::{HashMap, HashSet};

/// The linear strategy is simple - get the next task and queue
///         it for all hosts, then wait for the queue to drain before
///         moving on to the next task
pub struct LinearStrategy<'a> {
    tqm: &'a TaskQueueManager<'a>,
    inventory_manager: &'a InventoryManager,
    variable_manager: &'a VariableManager<'a>,
    host_cache: Vec<String>,
}

impl<'a> LinearStrategy<'a> {
    pub fn new(tqm: &'a TaskQueueManager) -> Self {
        LinearStrategy {
            tqm,
            inventory_manager: tqm.inventory_manager(),
            variable_manager: tqm.variable_manager(),
            host_cache: Vec::new(),
        }
    }

    fn set_host_cache(&mut self, play: &Play, refresh: bool) -> Result<()> {
        if !refresh && !self.host_cache.is_empty() {
            return Ok(());
        }

        // TODO: check ansible logic here
        let pattern = play.pattern();
        let limit = play.limit();

        self.host_cache = self
            .inventory_manager
            .filter_hosts(pattern, limit)?
            .iter()
            .map(|h| h.name().to_string())
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
    ) -> Result<Vec<(String, Task)>> {
        let mut state_task_per_host: HashMap<String, (HostState, Task)> = HashMap::new();
        let mut host_tasks: Vec<(String, Task)> = Vec::new();

        for host in hosts {
            let (state, task) = iterator.get_next_task_for_host(host, true)?;

            if let Some(BlockEntry::Task(task)) = task {
                // ansible assumes this is always a task, not a block?
                state_task_per_host.insert(host.name().to_string(), (state, task));
            } else {
                warn!(
                    "Unexpected block entry type for host {}: {:?}",
                    host.name(),
                    task
                )
            }
        }

        if state_task_per_host.is_empty() {
            return Ok(host_tasks);
        }

        let task_uuids: HashSet<&str> = state_task_per_host
            .values()
            .map(|(_, task)| task.uuid())
            .collect();

        let mut loop_cnt = 0;

        let mut cur_task: Option<Task> = None;

        while loop_cnt <= 1 {
            cur_task = iterator.get_current_task().map(|t| t.clone());

            if let Some(ref task) = cur_task {
                iterator.set_current_task_index(iterator.get_current_task_index() + 1);
                if task_uuids.contains(task.uuid()) {
                    break;
                }
            } else {
                loop_cnt += 1;
                iterator.set_current_task_index(0);
            }

            if loop_cnt > 1 {
                bail!("BUG: There seems to be a mismatch between tasks in PlayIterator and HostStates.");
            }
        }

        for (host_name, (state, task)) in state_task_per_host {
            if let Some(ref cur_task) = cur_task {
                if cur_task.uuid() == task.uuid() {
                    iterator.set_state_for_host(&host_name, state);
                    host_tasks.push((host_name, task.clone()));
                }
            }
        }

        Ok(host_tasks)
    }

    pub fn run(&mut self, iterator: &mut PlayIterator) -> Result<()> {
        self.set_host_cache(iterator.play(), false)?;
        let mut work_to_do = true;

        while work_to_do && !self.tqm.is_terminated() {
            debug!("getting the remaining hosts for this loop");
            let hosts_left = self.get_hosts_left();

            let callback_sent = false;
            work_to_do = false;

            let host_tasks = self.get_next_task_lockstep(hosts_left.clone(), iterator)?;

            let skip_rest = false;
            let choose_step = true;
            let any_errors_fatal = false;

            for (host, task) in host_tasks {
                if self.tqm.is_terminated() {
                    break;
                }
                debug!("getting variables");
                let host = self
                    .inventory_manager
                    .get_host(&host)
                    .ok_or(anyhow!("Host not found: {}", host))?;
                let task_vars = self.variable_manager.get_vars(
                    Some(iterator.play()),
                    Some(host),
                    Some(&task),
                    true,
                    true,
                );
            }
        }

        Ok(())
    }

    pub fn cleanup(&self) {}
}
