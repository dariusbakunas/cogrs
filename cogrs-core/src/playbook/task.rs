use crate::playbook::block::Block;
use crate::playbook::role::Role;

#[derive(Clone)]
pub enum Action {
    Module(String, String),
}

#[derive(Clone)]
pub struct Task {
    role: Option<Role>,
    action: Action,
    poll_interval: Option<u64>,
    async_val: Option<u64>,
    tags: Vec<String>,
    implicit: bool,
}

// TODO: add task builder

impl Task {
    fn new(
        action: &Action,
        role: Option<Role>,
        poll_interval: Option<u64>,
        async_val: Option<u64>,
        implicit: bool,
        tags: Vec<String>,
    ) -> Self {
        Self {
            action: action.clone(),
            role,
            poll_interval,
            async_val,
            implicit,
            tags,
        }
    }
}

pub struct TaskBuilder {
    action: Action,
    role: Option<Role>,
    poll_interval: Option<u64>,
    async_val: Option<u64>,
    implicit: bool,
    tags: Vec<String>,
}

impl TaskBuilder {
    pub fn new(action: Action) -> TaskBuilder {
        TaskBuilder {
            action,
            role: None,
            poll_interval: None,
            async_val: None,
            implicit: false,
            tags: vec![],
        }
    }

    pub fn role(mut self, role: Role) -> Self {
        self.role = Some(role);
        self
    }

    pub fn poll_interval(mut self, interval: Option<u64>) -> Self {
        self.poll_interval = interval;
        self
    }

    pub fn async_val(mut self, val: Option<u64>) -> Self {
        self.async_val = val;
        self
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn implicit(mut self, value: bool) -> Self {
        self.implicit = value;
        self
    }

    pub fn build(self) -> Task {
        Task::new(
            &self.action,
            self.role,
            self.poll_interval,
            self.async_val,
            self.implicit,
            self.tags,
        )
    }
}
