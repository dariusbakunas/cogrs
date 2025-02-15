use crate::constants::DEFAULT_GATHERING;
use crate::executor::failed_state::{FailedState, FailedStates};
use crate::executor::host_state::{HostState, IteratingState};
use crate::inventory::host::Host;
use crate::inventory::manager::InventoryManager;
use crate::playbook::block::{Block, BlockEntry};
use crate::playbook::play::Play;
use crate::playbook::task::{Action, Task, TaskBuilder};
use anyhow::Result;
use log::{debug, info};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::ops::BitAnd;

pub struct PlayIterator<'a> {
    blocks: Vec<BlockEntry>,
    batch_size: u32,
    host_states: HashMap<String, HostState>,
    end_play: bool,
    cur_task: u32,
    play: &'a Play,
}

impl<'a> PlayIterator<'a> {
    pub fn new(play: &'a Play) -> Self {
        PlayIterator {
            blocks: vec![],
            batch_size: 0,
            host_states: HashMap::new(),
            end_play: false,
            cur_task: 0,
            play,
        }
    }

    pub fn init(&mut self, inventory_manager: &InventoryManager) -> Result<()> {
        let mut setup_block = Block::new();
        let batch = inventory_manager.filter_hosts(self.play.get_pattern(), None)?;
        self.batch_size = batch.len() as u32;

        let mut setup_task_builder =
            TaskBuilder::new(Action::Module("setup".to_string(), "".to_string()));

        // Unless play is specifically tagged, gathering should 'always' run
        if self.play.get_tags().is_empty() {
            setup_task_builder = setup_task_builder.tags(vec!["always".to_string()]);
        }

        let setup_task = setup_task_builder.build();

        setup_block.add_to_block(BlockEntry::Task(setup_task));
        self.blocks.push(BlockEntry::Block(Box::new(setup_block)));

        for block in self.play.compile() {
            // TODO: filter tagged tasks
            if let BlockEntry::Block(block) = block {
                if block.has_any_entries() {
                    self.blocks.push(BlockEntry::Block(block));
                }
            } else if let BlockEntry::Task(task) = block {
                info!("Adding task to play iterator");
            }
        }

        for host in batch {
            let host_state = HostState::new(&self.blocks);
            self.host_states
                .insert(host.get_name().to_string(), host_state);

            // TODO: handle start_at_task option here
        }

        Ok(())
    }

    fn set_failed_state(&self, host_state: &mut HostState) {
        match host_state.get_run_state() {
            IteratingState::Setup => {
                host_state.set_fail_state(host_state.get_fail_state() | FailedState::Setup);
                host_state.set_run_state(IteratingState::Complete);
            }
            IteratingState::Tasks => {
                if let Some(mut child_state) = host_state.get_tasks_child_state().map(|s| s.clone())
                {
                    self.set_failed_state(&mut child_state);
                    host_state.set_tasks_child_state(Some(&child_state));
                } else {
                    host_state.set_fail_state(host_state.get_fail_state() | FailedState::Tasks);
                    if let Some(BlockEntry::Block(block)) = host_state.get_current_block() {
                        if block.has_rescue_entries() {
                            host_state.set_run_state(IteratingState::Rescue);
                        } else if block.has_always_entries() {
                            host_state.set_run_state(IteratingState::Always);
                        } else {
                            host_state.set_run_state(IteratingState::Complete);
                        }
                    }
                }
            }
            IteratingState::Rescue => {
                if let Some(mut child_state) =
                    host_state.get_rescue_child_state().map(|s| s.clone())
                {
                    self.set_failed_state(&mut child_state);
                    host_state.set_rescue_child_state(Some(&child_state));
                } else {
                    host_state.set_fail_state(host_state.get_fail_state() | FailedState::Rescue);
                    if let Some(BlockEntry::Block(block)) = host_state.get_current_block() {
                        if block.has_always_entries() {
                            host_state.set_run_state(IteratingState::Always);
                        } else {
                            host_state.set_run_state(IteratingState::Complete);
                        }
                    }
                }
            }
            IteratingState::Always => {
                if let Some(mut child_state) =
                    host_state.get_always_child_state().map(|s| s.clone())
                {
                    self.set_failed_state(&mut child_state);
                    host_state.set_always_child_state(Some(&child_state));
                } else {
                    host_state.set_fail_state(host_state.get_fail_state() | FailedState::Always);
                    host_state.set_run_state(IteratingState::Complete);
                }
            }
            _ => {}
        }
    }

    fn check_failed_state(&self, host_state: Option<&HostState>) -> bool {
        if let Some(host_state) = host_state {
            let run_state = host_state.get_run_state();
            let failed_state = host_state.get_fail_state();

            match run_state {
                IteratingState::Rescue => {
                    if self.check_failed_state(host_state.get_rescue_child_state()) {
                        return true;
                    }
                }
                IteratingState::Always => {
                    if self.check_failed_state(host_state.get_always_child_state()) {
                        return true;
                    }
                }
                IteratingState::Tasks => {
                    if self.check_failed_state(host_state.get_tasks_child_state()) {
                        if let Some(BlockEntry::Block(block)) = host_state.get_current_block() {
                            if block.has_rescue_entries() && failed_state & FailedState::Rescue == 0
                            {
                                return false;
                            }
                            return true;
                        }
                    }
                }
                _ => {}
            }

            if failed_state != FailedStates::new() {
                return match run_state {
                    IteratingState::Rescue => !(failed_state & FailedState::Rescue == 0),
                    IteratingState::Always => !(failed_state & FailedState::Always == 0),
                    _ => !(host_state.did_rescue() && failed_state & FailedState::Always == 0),
                };
            }
        }

        false
    }

    fn get_next_task_from_state(&self, host_state: &mut HostState) -> Result<Option<BlockEntry>> {
        // try and find the next task, given the current state.
        let mut task: Option<BlockEntry> = None;

        loop {
            let block = host_state.get_current_block();
            if block.is_none() {
                host_state.set_run_state(IteratingState::Complete);
                return Ok(None);
            }

            let block = block.unwrap();

            match host_state.get_run_state() {
                IteratingState::Setup => {
                    // First, we check to see if we were pending setup. If not, this is
                    // the first trip through IteratingStates.SETUP, so we set the pending_setup
                    // flag and try to determine if we do in fact want to gather facts for
                    // the specified host.
                    if !host_state.is_pending_setup() {
                        host_state.set_pending_setup(true);
                        let gathering = DEFAULT_GATHERING;
                        let implied = self.play.gather_facts().unwrap_or(true);

                        let should_gather_facts = match gathering {
                            "implicit" if implied => true,
                            "explicit" if self.play.gather_facts().is_some_and(|g| g) => true,
                            "smart" => true, // TODO: handle smart
                            _ => false,
                        };

                        if should_gather_facts {
                            let setup_block = self.blocks[0].clone();
                            if let BlockEntry::Block(block) = setup_block {
                                if let Some(block_entry) = block.get_block_entry(0) {
                                    task = Some(block_entry.clone());
                                }
                            }
                        }
                    } else {
                        // This is the second trip through IteratingStates.SETUP, so we clear
                        // the flag and move onto the next block in the list while setting
                        // the run state to IteratingStates.TASKS
                        host_state.set_pending_setup(false);
                        host_state.set_run_state(IteratingState::Tasks);
                        if !host_state.did_start_at_task() {
                            host_state
                                .set_current_block_index(host_state.get_current_block_index() + 1);
                            host_state.set_current_regular_task_index(0);
                            host_state.set_current_rescue_task_index(0);
                            host_state.set_current_always_task_index(0);
                            host_state.set_current_handler_task_index(0);
                            host_state.set_always_child_state(None);
                            host_state.set_tasks_child_state(None);
                            host_state.set_rescue_child_state(None);
                        }
                    }
                }
                IteratingState::Tasks => {
                    // TODO: I don't see how this could still be pending setup?
                    if host_state.is_pending_setup() {
                        host_state.set_pending_setup(false);
                    }

                    // First, we check for a child task state that is not failed, and if we
                    // have one recurse into it for the next task. If we're done with the child
                    // state, we clear it and drop back to getting the next task from the list.
                    if let Some(mut task_child_state) =
                        host_state.get_tasks_child_state().map(|s| s.clone())
                    {
                        task = self.get_next_task_from_state(&mut task_child_state)?;
                        if self.check_failed_state(Some(&task_child_state)) {
                            // failed child state, so clear it and move into the rescue portion
                            host_state.set_tasks_child_state(None);
                            self.set_failed_state(host_state);
                        } else {
                            // get the next task recursively
                            if task.is_none()
                                || task_child_state.get_run_state() == IteratingState::Complete
                            {
                                // we're done with the child state, so clear it and continue
                                // back to the top of the loop to get the next task
                                host_state.set_tasks_child_state(None);
                                continue;
                            }
                        }
                    } else {
                        // First here, we check to see if we've failed anywhere down the chain
                        // of states we have, and if so we move onto the rescue portion. Otherwise,
                        // we check to see if we've moved past the end of the list of tasks. If so,
                        // we move into the always portion of the block, otherwise we get the next
                        // task from the list.
                    }
                }
                IteratingState::Rescue => {}
                IteratingState::Always => {}
                IteratingState::Handlers => {}
                IteratingState::Complete => {}
            }

            // if something above set the task, break out of the loop now
            if task.is_some() {}
        }

        Ok(task)
    }

    pub fn get_next_task_for_host(
        &mut self,
        host: &Host,
        peek: bool,
    ) -> Result<(HostState, Option<Task>)> {
        debug!("Getting next task for host: {}", host.get_name());
        let mut host_state = self
            .host_states
            .get(host.get_name())
            .ok_or(anyhow::format_err!(
                "Host state {} not found in play iterator",
                host.get_name()
            ))?
            .clone();

        if host_state.is_complete() {
            debug!("Host {} is done iterating, returning", host.get_name());
            return Ok((host_state.clone(), None));
        }

        let task = self.get_next_task_from_state(&mut host_state)?;

        if !peek {
            self.host_states
                .insert(host.get_name().to_string(), host_state);
        }

        //debug!("Done getting next task for host {}, task: {}", host.get_name(), task);
        todo!()
    }

    pub fn get_batch_size(&self) -> u32 {
        self.batch_size
    }

    pub fn get_play(&self) -> &Play {
        self.play
    }
}
