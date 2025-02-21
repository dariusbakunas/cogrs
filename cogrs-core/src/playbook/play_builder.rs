use crate::playbook::block::{Block, BlockEntry};
use crate::playbook::handler::Handler;
use crate::playbook::play::Play;
use crate::playbook::role::Role;
use crate::playbook::task::Task;
use crate::strategy::Strategy;
use crate::vars::variable::Variable;
use indexmap::IndexMap;

const GATHER_TIMEOUT_DEFAULT: u32 = 10;

pub struct PlayBuilder {
    any_errors_fatal: bool,
    become_exe: Option<String>,
    become_flags: Option<String>,
    become_method: Option<String>,
    become_user: Option<String>,
    check_mode: bool,
    connection: String,
    diff: bool,
    finalized: bool,
    force_handlers: bool,
    gather_facts: Option<bool>,
    gather_subset: Vec<String>,
    gather_timeout: u32,
    handlers: Vec<Handler>,
    limit: Option<String>,
    name: String,
    no_log: bool,
    pattern: String,
    post_tasks: Vec<Block>,
    pre_tasks: Vec<Block>,
    roles: Vec<Role>,
    strategy: Strategy,
    tags: Vec<String>,
    tasks: Vec<Block>,
    throttle: u32,
    timeout: u32,
    use_become: bool,
    vars: IndexMap<String, Variable>,
    vars_files: Vec<String>,
}

impl PlayBuilder {
    pub fn new(name: &str, roles: &[Role]) -> PlayBuilder {
        PlayBuilder {
            any_errors_fatal: false,
            become_exe: None,
            become_flags: None,
            become_method: None,
            become_user: None,
            check_mode: false,
            connection: String::from("ssh"),
            diff: false,
            finalized: false,
            force_handlers: false,
            gather_facts: None,
            gather_subset: Vec::new(),
            gather_timeout: GATHER_TIMEOUT_DEFAULT,
            handlers: Vec::new(),
            limit: None,
            name: String::from(name),
            no_log: false,
            pattern: String::from("all"),
            post_tasks: Vec::new(),
            pre_tasks: Vec::new(),
            roles: roles.to_vec(),
            strategy: Strategy::Linear,
            tags: Vec::new(),
            tasks: Vec::new(),
            throttle: 0,
            timeout: 0,
            use_become: false,
            vars: IndexMap::new(),
            vars_files: Vec::new(),
        }
    }

    pub fn use_become(mut self, value: bool) -> Self {
        self.use_become = value;
        self
    }

    pub fn become_user(mut self, user: &str) -> Self {
        self.become_user = Some(user.to_string());
        self
    }

    pub fn check_mode(mut self, value: bool) -> Self {
        self.check_mode = value;
        self
    }

    pub fn connection(mut self, connection: &str) -> Self {
        self.connection = connection.to_string();
        self
    }

    pub fn diff(mut self, value: bool) -> Self {
        self.diff = value;
        self
    }

    pub fn gather_facts(mut self, value: bool) -> Self {
        self.gather_facts = Some(value);
        self
    }

    pub fn gather_subset(mut self, subset: Vec<String>) -> Self {
        self.gather_subset = subset;
        self
    }

    pub fn gather_timeout(mut self, timeout: u32) -> Self {
        self.gather_timeout = timeout;
        self
    }

    pub fn no_log(mut self, value: bool) -> Self {
        self.no_log = value;
        self
    }

    pub fn strategy(mut self, strategy: Strategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn tasks(mut self, tasks: &[Task]) -> Self {
        let mut blocks: Vec<Block> = Vec::new();
        for task in tasks {
            let mut block = Block::new();
            block.set_is_implicit(true);
            block.add_to_block(BlockEntry::Task(task.clone()));
            blocks.push(block);
        }
        self.tasks = blocks;
        self
    }

    pub fn throttle(mut self, throttle: u32) -> Self {
        self.throttle = throttle;
        self
    }

    pub fn timeout(mut self, timeout: u32) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn pattern(mut self, pattern: &str) -> Self {
        self.pattern = pattern.to_string();
        self
    }

    pub fn limit(mut self, limit: Option<&str>) -> Self {
        self.limit = limit.map(|l| String::from(l));
        self
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn finalized(mut self, value: bool) -> Self {
        self.finalized = value;
        self
    }

    pub fn build(self) -> Play {
        Play::new(
            self.any_errors_fatal,
            self.become_exe,
            self.become_flags,
            self.become_method,
            self.become_user,
            self.check_mode,
            self.connection,
            self.diff,
            self.finalized,
            self.force_handlers,
            self.gather_facts,
            self.gather_subset,
            self.gather_timeout,
            self.handlers,
            self.limit,
            self.name,
            self.no_log,
            self.pattern,
            self.post_tasks,
            self.pre_tasks,
            self.roles,
            self.strategy,
            self.tags,
            self.tasks,
            self.throttle,
            self.timeout,
            self.use_become,
            self.vars,
            self.vars_files,
        )
    }
}
