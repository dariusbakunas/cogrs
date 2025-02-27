use crate::playbook::role::Role;
use crate::utils::get_unique_id;
use std::fmt;
use std::fmt::Formatter;

#[derive(Clone, Debug)]
pub enum Action {
    Module(String, Option<String>),
    Meta(String),
    Handler(String),
}

#[derive(Clone, Debug)]
pub struct Task {
    uuid: String,
    name: String,
    role: Option<Role>,
    action: Action,
    poll_interval: Option<u64>,
    async_val: Option<u64>,
    tags: Vec<String>,
    implicit: bool,
    throttle: usize,
    run_once: bool,
    connection: String,
}

// TODO: add task builder

impl Task {
    fn new(
        name: String,
        action: &Action,
        role: Option<Role>,
        poll_interval: Option<u64>,
        async_val: Option<u64>,
        implicit: bool,
        tags: Vec<String>,
        connection: String,
    ) -> Self {
        Self {
            name,
            uuid: get_unique_id(false),
            action: action.clone(),
            role,
            poll_interval,
            async_val,
            implicit,
            tags,
            throttle: 0, // TODO: where do we  get throttle
            run_once: false,
            connection,
        }
    }

    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    pub fn connection(&self) -> &str {
        &self.connection
    }

    pub fn action(&self) -> &Action {
        &self.action
    }

    pub fn role(&self) -> Option<&Role> {
        self.role.as_ref()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn poll_interval(&self) -> Option<u64> {
        self.poll_interval
    }

    pub fn async_val(&self) -> Option<u64> {
        self.async_val
    }

    pub fn throttle(&self) -> usize {
        self.throttle
    }

    pub fn run_once(&self) -> bool {
        self.run_once
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.action {
            Action::Module(name, _) => {
                write!(f, "TASK: {}", name)
            }
            Action::Meta(_) => {
                write!(f, "META TASK")
            }
            Action::Handler(_) => {
                write!(f, "HANDLER TASK")
            }
        }
    }
}

pub struct TaskBuilder {
    name: String,
    action: Action,
    role: Option<Role>,
    poll_interval: Option<u64>,
    async_val: Option<u64>,
    implicit: bool,
    tags: Vec<String>,
    connection: String,
}

impl TaskBuilder {
    pub fn new(name: &str, connection: &str, action: Action) -> TaskBuilder {
        TaskBuilder {
            name: name.to_string(),
            action,
            role: None,
            poll_interval: None,
            async_val: None,
            implicit: false,
            tags: Vec::new(),
            connection: connection.to_string(),
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
            self.name,
            &self.action,
            self.role,
            self.poll_interval,
            self.async_val,
            self.implicit,
            self.tags,
            self.connection,
        )
    }
}
