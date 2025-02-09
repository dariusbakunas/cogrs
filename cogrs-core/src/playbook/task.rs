use crate::playbook::role::Role;

#[derive(Clone)]
pub enum Action {
    Module(String, String),
}

#[derive(Clone)]
pub struct Task {
    name: String,
    role: Option<Role>,
    action: Action,
    poll_interval: Option<u64>,
    async_val: Option<u64>,
}

impl Task {
    pub fn new(
        name: &str,
        action: &Action,
        role: Option<Role>,
        poll_interval: Option<u64>,
        async_val: Option<u64>,
    ) -> Self {
        Self {
            name: name.to_string(),
            action: action.clone(),
            role,
            poll_interval,
            async_val,
        }
    }
}
