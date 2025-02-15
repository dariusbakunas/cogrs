use crate::executor::failed_state::FailedStates;
use crate::playbook::block::BlockEntry;
use std::cmp::PartialEq;
use std::ops::{BitAnd, BitOr};

#[derive(Debug, PartialEq, Clone)]
pub enum IteratingState {
    Setup,
    Tasks,
    Rescue,
    Always,
    Handlers,
    Complete,
}

#[derive(Clone)]
pub struct HostState {
    blocks: Vec<BlockEntry>,
    update_handlers: bool,
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
    pub fn new(blocks: &[BlockEntry]) -> Self {
        HostState {
            blocks: blocks.to_vec(),
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

    pub fn get_run_state(&self) -> IteratingState {
        self.run_state.clone()
    }

    pub fn get_fail_state(&self) -> FailedStates {
        self.fail_state.clone()
    }

    pub fn set_run_state(&mut self, state: IteratingState) {
        self.run_state = state;
    }

    pub fn get_current_block(&self) -> Option<&BlockEntry> {
        self.blocks.get(self.curr_block)
    }

    pub fn get_current_block_index(&self) -> usize {
        self.curr_block
    }

    pub fn set_current_block_index(&mut self, index: usize) {
        self.curr_block = index;
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

    pub fn get_rescue_child_state(&self) -> Option<&HostState> {
        self.rescue_child_state.as_ref().map(|s| &**s)
    }

    pub fn set_rescue_child_state(&mut self, state: Option<&HostState>) {
        self.rescue_child_state = state.map(|s| Box::new(s.clone()));
    }

    pub fn get_always_child_state(&self) -> Option<&HostState> {
        self.always_child_state.as_ref().map(|s| &**s)
    }

    pub fn set_always_child_state(&mut self, state: Option<&HostState>) {
        self.always_child_state = state.map(|s| Box::new(s.clone()));
    }

    pub fn get_tasks_child_state(&self) -> Option<&HostState> {
        self.tasks_child_state.as_ref().map(|s| &**s)
    }

    pub fn set_fail_state(&mut self, state: FailedStates) {
        self.fail_state = state;
    }

    pub fn get_block_count(&self) -> usize {
        self.blocks.len()
    }

    pub fn did_start_at_task(&self) -> bool {
        self.did_start_at_task
    }
}
