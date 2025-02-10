pub enum IteratingState {
    Setup,
    Tasks,
    Rescue,
    Always,
    Handlers,
    Complete,
}

pub enum FailedState {
    None,
    Setup,
    Tasks,
    Rescue,
    Always,
    Handlers,
}

pub struct HostState {
    update_handlers: bool,
    pending_setup: bool,
    did_rescue: bool,
    did_start_at_task: bool,
    run_state: IteratingState,
    fail_state: FailedState,
}

impl HostState {
    pub fn new() -> Self {
        HostState {
            update_handlers: false,
            pending_setup: false,
            did_rescue: false,
            did_start_at_task: false,
            run_state: IteratingState::Setup,
            fail_state: FailedState::None,
        }
    }
}
