use crate::playbook::block::{Block, BlockEntry};
use crate::playbook::role::Role;
use crate::playbook::task::{Action, Task, TaskBuilder};
use crate::strategy::Strategy;

const GATHER_TIMEOUT_DEFAULT: u32 = 10;

#[derive(Clone)]
pub struct Play {
    pub name: String,
    tasks: Vec<BlockEntry>,
    pre_tasks: Vec<BlockEntry>,
    post_tasks: Vec<BlockEntry>,
    roles: Vec<Role>,
    use_become: bool,
    force_handlers: bool,
    become_user: Option<String>,
    check_mode: bool,
    connection: String,
    diff: bool,
    gather_facts: bool,
    gather_subset: Vec<String>,
    gather_timeout: u32,
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
        tasks: Vec<BlockEntry>,
        pre_tasks: Vec<BlockEntry>,
        post_tasks: Vec<BlockEntry>,
        roles: Vec<Role>,
        use_become: bool,
        become_user: Option<String>,
        force_handlers: bool,
        check_mode: bool,
        connection: String,
        diff: bool,
        gather_facts: bool,
        gather_subset: Vec<String>,
        gather_timeout: u32,
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
            pre_tasks,
            post_tasks,
            roles,
            use_become,
            become_user,
            force_handlers,
            check_mode,
            connection,
            diff,
            gather_facts,
            gather_subset,
            gather_timeout,
            no_log,
            strategy,
            throttle,
            timeout,
            pattern,
            tags,
        }
    }

    pub fn builder(name: &str, roles: &[Role]) -> PlayBuilder {
        PlayBuilder::new(name, roles)
    }

    pub fn get_pattern(&self) -> &str {
        self.pattern.as_str()
    }

    pub fn get_tags(&self) -> &Vec<String> {
        &self.tags
    }

    pub fn get_strategy(&self) -> &Strategy {
        &self.strategy
    }

    fn compile_roles(&self) -> Vec<BlockEntry> {
        let mut blocks: Vec<BlockEntry> = Vec::new();

        // TODO: compile_rows

        blocks
    }

    pub fn compile(&self) -> Vec<BlockEntry> {
        let mut blocks: Vec<BlockEntry> = Vec::new();

        // create a block containing a single flush handlers meta
        // task, so we can be sure to run handlers at certain points
        // of the playbook execution
        let mut flush_block = Block::new();

        let meta_task = TaskBuilder::new(Action::Module(
            "meta".to_string(),
            "flush_handlers".to_string(),
        ))
        .implicit(true)
        .build();

        if self.tags.is_empty() {
            flush_block.add_to_block(BlockEntry::Task(meta_task));
        } else {
            // TODO: evaluate tags
        };

        if self.force_handlers {
            let noop_task =
                TaskBuilder::new(Action::Module("meta".to_string(), "noop".to_string()))
                    .implicit(true)
                    .build();

            // TODO: add remaining blocks
        }

        blocks.extend(self.pre_tasks.clone());
        blocks.push(BlockEntry::Block(Box::new(flush_block.clone())));
        blocks.extend(self.compile_roles());
        blocks.extend(self.tasks.clone());
        blocks.push(BlockEntry::Block(Box::new(flush_block.clone())));
        blocks.extend(self.post_tasks.clone());
        blocks.push(BlockEntry::Block(Box::new(flush_block)));

        blocks
    }
}

pub struct PlayBuilder {
    name: String,
    tasks: Vec<BlockEntry>,
    pre_tasks: Vec<BlockEntry>,
    post_tasks: Vec<BlockEntry>,
    roles: Vec<Role>,
    use_become: bool,
    become_user: Option<String>,
    force_handlers: bool,
    check_mode: bool,
    connection: String,
    diff: bool,
    gather_facts: bool,
    gather_subset: Vec<String>,
    gather_timeout: u32,
    no_log: bool,
    strategy: Strategy,
    throttle: u32,
    timeout: u32,
    pattern: String,
    tags: Vec<String>,
}

impl PlayBuilder {
    pub fn new(name: &str, roles: &[Role]) -> PlayBuilder {
        PlayBuilder {
            name: String::from(name),
            tasks: Vec::new(),
            pre_tasks: Vec::new(),
            post_tasks: Vec::new(),
            roles: roles.to_vec(),
            use_become: false,
            become_user: None,
            force_handlers: false,
            check_mode: false,
            connection: String::from("ssh"),
            diff: false,
            gather_facts: false,
            gather_subset: vec![],
            gather_timeout: GATHER_TIMEOUT_DEFAULT,
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
        let mut blocks: Vec<BlockEntry> = Vec::new();
        for task in tasks {
            let mut block = Block::new();
            block.set_is_implicit(true);
            block.add_to_block(BlockEntry::Task(task.clone()));
            blocks.push(BlockEntry::Block(Box::new(block)));
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
            self.pre_tasks,
            self.post_tasks,
            self.roles,
            self.use_become,
            self.become_user,
            self.force_handlers,
            self.check_mode,
            self.connection,
            self.diff,
            self.gather_facts,
            self.gather_subset,
            self.gather_timeout,
            self.no_log,
            self.strategy,
            self.throttle,
            self.timeout,
            self.pattern,
            self.tags,
        )
    }
}
