use crate::playbook::block::Block;
use crate::playbook::role::Role;
use crate::playbook::task::Task;

#[derive(Clone)]
pub enum Strategy {
    Linear,
    Free,
}

#[derive(Clone)]
pub struct Play {
    pub name: String,
    tasks: Vec<Task>,
    roles: Vec<Role>,
    use_become: bool,
    become_user: Option<String>,
    check_mode: bool,
    connection: String,
    diff: bool,
    gather_facts: bool,
    no_log: bool,
    strategy: Strategy,
    throttle: u32,
    timeout: u32,
    pattern: String,
    tags: Vec<String>,
}

impl Play {
    #[allow(clippy::too_many_arguments)]
    fn new(
        name: String,
        tasks: Vec<Task>,
        roles: Vec<Role>,
        use_become: bool,
        become_user: Option<String>,
        check_mode: bool,
        connection: String,
        diff: bool,
        gather_facts: bool,
        no_log: bool,
        strategy: Strategy,
        throttle: u32,
        timeout: u32,
        pattern: String,
        tags: Vec<String>,
    ) -> Self {
        Play {
            name,
            tasks,
            roles,
            use_become,
            become_user,
            check_mode,
            connection,
            diff,
            gather_facts,
            no_log,
            strategy,
            throttle,
            timeout,
            pattern,
            tags,
        }
    }

    pub fn builder(name: &str, tasks: &[Task], roles: &[Role]) -> PlayBuilder {
        PlayBuilder::new(name, tasks, roles)
    }

    pub fn get_pattern(&self) -> &str {
        self.pattern.as_str()
    }

    pub fn get_tags(&self) -> &Vec<String> {
        &self.tags
    }

    pub fn compile(&self) -> Vec<Block> {
        let blocks = Vec::new();

        blocks
    }
}

pub struct PlayBuilder {
    name: String,
    tasks: Vec<Task>,
    roles: Vec<Role>,
    use_become: bool,
    become_user: Option<String>,
    check_mode: bool,
    connection: String,
    diff: bool,
    gather_facts: bool,
    no_log: bool,
    strategy: Strategy,
    throttle: u32,
    timeout: u32,
    pattern: String,
    tags: Vec<String>,
}

impl PlayBuilder {
    pub fn new(name: &str, tasks: &[Task], roles: &[Role]) -> PlayBuilder {
        PlayBuilder {
            name: String::from(name),
            tasks: tasks.to_vec(),
            roles: roles.to_vec(),
            use_become: false,
            become_user: None,
            check_mode: false,
            connection: String::from("ssh"),
            diff: false,
            gather_facts: false,
            no_log: false,
            strategy: Strategy::Linear,
            throttle: 0,
            timeout: 0,
            pattern: String::from(""),
            tags: vec![],
        }
    }

    pub fn use_become(mut self, value: bool) -> Self {
        self.use_become = value;
        self
    }

    pub fn become_user(mut self, user: String) -> Self {
        self.become_user = Some(user);
        self
    }

    pub fn check_mode(mut self, value: bool) -> Self {
        self.check_mode = value;
        self
    }

    pub fn connection(mut self, connection: String) -> Self {
        self.connection = connection;
        self
    }

    pub fn diff(mut self, value: bool) -> Self {
        self.diff = value;
        self
    }

    pub fn gather_facts(mut self, value: bool) -> Self {
        self.gather_facts = value;
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

    pub fn throttle(mut self, throttle: u32) -> Self {
        self.throttle = throttle;
        self
    }

    pub fn timeout(mut self, timeout: u32) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn pattern(mut self, pattern: String) -> Self {
        self.pattern = pattern;
        self
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn build(self) -> Play {
        Play::new(
            self.name,
            self.tasks,
            self.roles,
            self.use_become,
            self.become_user,
            self.check_mode,
            self.connection,
            self.diff,
            self.gather_facts,
            self.no_log,
            self.strategy,
            self.throttle,
            self.timeout,
            self.pattern,
            self.tags,
        )
    }
}
