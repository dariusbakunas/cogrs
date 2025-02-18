use crate::executor::host_state::HostState;
use crate::executor::play_iterator::PlayIterator;
use crate::executor::task_executor::TaskExecutor;
use crate::executor::task_queue_manager::TaskQueueManager;
use crate::executor::WorkerMessage::WorkerMessage;
use crate::inventory::host::Host;
use crate::inventory::manager::InventoryManager;
use crate::playbook::block::BlockEntry;
use crate::playbook::play::Play;
use crate::playbook::task::{Action, Task};
use crate::vars::manager::VariableManager;
use crate::vars::variable::Variable;
use anyhow::{anyhow, bail, Result};
use cogrs_plugins::callback::EventType;
use log::{debug, warn};
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc;
use tokio::time::{self, Duration};

/// The linear strategy is simple - get the next task and queue
///         it for all hosts, then wait for the queue to drain before
///         moving on to the next task
pub struct LinearStrategy<'a> {
    tqm: &'a mut TaskQueueManager,
    inventory_manager: &'a InventoryManager,
    variable_manager: &'a VariableManager,
    host_cache: Vec<String>,
    blocked_hosts: HashMap<String, bool>,
    cur_worker: usize,
    pending_results: u32,
}

async fn results_thread(mut receiver: mpsc::Receiver<WorkerMessage>) {
    while let Some(msg) = receiver.recv().await {
        match msg {
            WorkerMessage::Callback(msg) => {
                debug!("received callback from worker: {}", msg);
            }
            WorkerMessage::Display(_) => {}
            WorkerMessage::Prompt(_) => {}
        }
    }
}

impl<'a> LinearStrategy<'a> {
    pub fn new(
        tqm: &'a mut TaskQueueManager,
        inventory_manager: &'a InventoryManager,
        variable_manager: &'a VariableManager,
    ) -> Self {
        LinearStrategy {
            tqm,
            inventory_manager,
            variable_manager,
            host_cache: Vec::new(),
            blocked_hosts: HashMap::new(),
            cur_worker: 0,
            pending_results: 0,
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

    pub async fn run(&mut self, iterator: &mut PlayIterator) -> Result<()> {
        self.set_host_cache(iterator.play(), false)?;
        let mut work_to_do = true;
        let mut callback_sent = false;

        // TODO: how big of a channel do we want?
        let (sender, receiver) = mpsc::channel(100);
        let reader = tokio::spawn(results_thread(receiver));

        while work_to_do && !self.tqm.is_terminated() {
            debug!("getting the remaining hosts for this loop");
            let hosts_left = self.get_hosts_left();

            let mut callback_sent = false;
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

                if let Action::Meta(action) = task.action() {
                    // TODO: handle meta actions
                } else {
                    if !callback_sent {
                        if let Action::Handler(_) = task.action() {
                            // TODO: send args to callback
                            self.tqm
                                .emit_event(EventType::PlaybookOnHandlerTaskStart, None)
                                .await
                        } else {
                            self.tqm
                                .emit_event(EventType::PlaybookOnTaskStart, None)
                                .await
                        }

                        callback_sent = true
                    }

                    self.blocked_hosts.insert(host.name().to_string(), true);
                    self.queue_task(host, &task, task_vars, sender.clone())
                        .await?;
                }
            }
        }

        reader.await?;

        Ok(())
    }

    fn spawn_new_worker(
        &mut self,
        worker_index: usize,
        sender: mpsc::Sender<WorkerMessage>,
        host: &Host,
        task: &Task,
    ) -> Result<()> {
        // TODO: figure out what needs to be cloned and what needs Arc
        let host = host.clone();
        let task = task.clone();

        let new_worker = tokio::spawn(async move {
            let executor = TaskExecutor::new();
            executor.run(&host, &task);
        });
        self.tqm.set_worker(worker_index, new_worker);
        Ok(())
    }

    /// handles queueing the task up to be sent to a worker
    async fn queue_task(
        &mut self,
        host: &Host,
        task: &Task,
        task_vars: HashMap<String, Variable>,
        sender: mpsc::Sender<WorkerMessage>,
    ) -> Result<()> {
        debug!("entering queue_task() for {}/{}", host.name(), task);

        let mut queued = false;
        let starting_worker = self.cur_worker;

        // Determine the "rewind point" of the worker list. This means we start
        // iterating over the list of workers until the end of the list is found.
        // Normally, that is simply the length of the workers list (as determined
        // by the forks or serial setting), however a task/block/play may "throttle"
        // that limit down.
        let mut rewind_point = self.tqm.forks();
        let throttle = task.throttle();

        if throttle > 0 {
            if task.run_once() {
                debug!(
                    "Ignoring 'throttle' as 'run_once' is also set for '{}'",
                    task.name()
                );
            } else {
                if throttle <= rewind_point {
                    debug!("task: {}, throttle: {}", task.name(), throttle);
                    rewind_point = throttle;
                }
            }
        }

        loop {
            if self.cur_worker >= rewind_point {
                self.cur_worker = 0;
            }

            let worker = self.tqm.get_worker(self.cur_worker);

            if let Some(worker) = worker {
                if !worker.is_finished() {
                    self.cur_worker += 1;
                } else {
                    self.spawn_new_worker(self.cur_worker, sender.clone(), host, task)?;
                    queued = true;
                }
            } else {
                self.spawn_new_worker(self.cur_worker, sender.clone(), host, task)?;
                queued = true;
            }

            self.cur_worker += 1;

            if self.cur_worker >= rewind_point {
                self.cur_worker = 0;
            }

            if queued {
                break;
            } else if self.cur_worker == starting_worker {
                time::sleep(Duration::from_micros(100)).await;
            }
        }

        self.pending_results += 1;

        Ok(())
    }

    pub fn cleanup(&self) {}
}
