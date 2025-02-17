use crate::executor::failed_state::FailedStates;
use crate::playbook::block::{Block, BlockEntry};
use std::cmp::PartialEq;

#[derive(Debug, PartialEq, Clone)]
pub enum IteratingState {
    Setup,
    Tasks,
    Rescue,
    Always,
    Handlers,
    Complete,
}

#[derive(Clone, Debug)]
pub struct HostState {
    name: String,
    blocks: Vec<Block>,
    update_handlers: bool,
    handler_notifications: Vec<String>,
    pending_setup: bool,
    did_rescue: bool,
    did_start_at_task: bool,
    run_state: IteratingState,
    fail_state: FailedStates,
    curr_block: usize,
    curr_regular_task: usize,
    curr_rescue_task: usize,
    curr_always_task: usize,
    curr_handler_task: usize,
    tasks_child_state: Option<Box<HostState>>,
    rescue_child_state: Option<Box<HostState>>,
    always_child_state: Option<Box<HostState>>,
}

impl HostState {
    pub fn new(name: &str, blocks: &[Block]) -> Self {
        HostState {
            name: name.to_string(),
            blocks: blocks.to_vec(),
            handler_notifications: Vec::new(),
            update_handlers: false,
            pending_setup: false,
            did_rescue: false,
            did_start_at_task: false,
            run_state: IteratingState::Setup,
            fail_state: FailedStates::new(),
            curr_block: 0,
            curr_regular_task: 0,
            curr_rescue_task: 0,
            curr_always_task: 0,
            curr_handler_task: 0,
            tasks_child_state: None,
            rescue_child_state: None,
            always_child_state: None,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn has_handler_notifications(&self) -> bool {
        !self.handler_notifications.is_empty()
    }

    pub fn did_rescue(&self) -> bool {
        self.did_rescue
    }

    pub fn is_complete(&self) -> bool {
        self.run_state == IteratingState::Complete
    }

    pub fn is_pending_setup(&self) -> bool {
        self.pending_setup
    }

    pub fn set_pending_setup(&mut self, pending: bool) {
        self.pending_setup = pending;
    }

    pub fn run_state(&self) -> IteratingState {
        self.run_state.clone()
    }

    pub fn fail_state(&self) -> FailedStates {
        self.fail_state.clone()
    }

    pub fn set_run_state(&mut self, state: IteratingState) {
        self.run_state = state;
    }

    pub fn current_block(&self) -> Option<&Block> {
        self.blocks.get(self.curr_block)
    }

    pub fn current_block_index(&self) -> usize {
        self.curr_block
    }

    pub fn current_always_task_index(&self) -> usize {
        self.curr_always_task
    }

    pub fn set_current_block_index(&mut self, index: usize) {
        self.curr_block = index;
    }

    pub fn current_regular_task_index(&self) -> usize {
        self.curr_regular_task
    }

    pub fn set_current_regular_task_index(&mut self, index: usize) {
        self.curr_regular_task = index;
    }

    pub fn set_current_rescue_task_index(&mut self, index: usize) {
        self.curr_rescue_task = index;
    }

    pub fn set_current_always_task_index(&mut self, index: usize) {
        self.curr_always_task = index;
    }

    pub fn set_current_handler_task_index(&mut self, index: usize) {
        self.curr_handler_task = index;
    }

    pub fn set_tasks_child_state(&mut self, state: Option<&HostState>) {
        self.tasks_child_state = state.map(|s| Box::new(s.clone()));
    }

    pub fn rescue_child_state(&self) -> Option<&HostState> {
        self.rescue_child_state.as_ref().map(|s| &**s)
    }

    pub fn set_rescue_child_state(&mut self, state: Option<&HostState>) {
        self.rescue_child_state = state.map(|s| Box::new(s.clone()));
    }

    pub fn always_child_state(&self) -> Option<&HostState> {
        self.always_child_state.as_ref().map(|s| &**s)
    }

    pub fn set_always_child_state(&mut self, state: Option<&HostState>) {
        self.always_child_state = state.map(|s| Box::new(s.clone()));
    }

    pub fn tasks_child_state(&self) -> Option<&HostState> {
        self.tasks_child_state.as_ref().map(|s| &**s)
    }

    pub fn set_fail_state(&mut self, state: FailedStates) {
        self.fail_state = state;
    }

    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    pub fn did_start_at_task(&self) -> bool {
        self.did_start_at_task
    }

    pub fn set_did_rescue(&mut self, did_rescue: bool) {
        self.did_rescue = did_rescue;
    }
}
