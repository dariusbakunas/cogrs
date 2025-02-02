use crate::task::Task;

#[derive(Clone)]
pub enum Strategy {
    Linear,
    Free,
}

#[derive(Clone)]
pub struct Play {
    pub name: String,
    tasks: Vec<Task>,
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
}

impl Play {
    #[allow(clippy::too_many_arguments)]
    fn new(
        name: String,
        tasks: Vec<Task>,
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
    ) -> Self {
        Play {
            name,
            tasks,
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
        }
    }

    pub fn builder(name: &str, tasks: &[Task]) -> PlayBuilder {
        PlayBuilder::new(name, tasks)
    }
}

pub struct PlayBuilder {
    name: String,
    tasks: Vec<Task>,
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
}

impl PlayBuilder {
    pub fn new(name: &str, tasks: &[Task]) -> PlayBuilder {
        PlayBuilder {
            name: String::from(name),
            tasks: tasks.to_vec(),
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

    pub fn build(self) -> Play {
        Play::new(
            self.name,
            self.tasks,
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
        )
    }
}
