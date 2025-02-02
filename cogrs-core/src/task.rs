#[derive(Clone)]
pub enum Action {
    Module(String, String),
}

#[derive(Clone)]
pub struct Task {
    name: String,
    action: Action,
    poll_interval: Option<u64>,
    async_val: Option<u64>,
}

impl Task {
    pub fn new(
        name: &str,
        action: &Action,
        poll_interval: Option<u64>,
        async_val: Option<u64>,
    ) -> Self {
        Self {
            name: name.to_string(),
            action: action.clone(),
            poll_interval,
            async_val,
        }
    }
}
