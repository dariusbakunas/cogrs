use crate::playbook::block::{Block, BlockEntry};
use crate::playbook::handler::Handler;
use crate::playbook::play_builder::PlayBuilder;
use crate::playbook::role::Role;
use crate::playbook::task::{Action, Task, TaskBuilder};
use crate::strategy::Strategy;
use crate::vars::variable::Variable;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Play {
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
    vars: HashMap<String, Variable>,
    vars_files: Vec<String>,
}

impl Play {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
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
        vars: HashMap<String, Variable>,
        vars_files: Vec<String>,
    ) -> Self {
        Play {
            any_errors_fatal,
            become_exe,
            become_flags,
            become_method,
            become_user,
            check_mode,
            connection,
            diff,
            finalized,
            force_handlers,
            gather_facts,
            gather_subset,
            gather_timeout,
            handlers,
            limit,
            name,
            no_log,
            pattern,
            post_tasks,
            pre_tasks,
            roles,
            strategy,
            tags,
            tasks,
            throttle,
            timeout,
            use_become,
            vars,
            vars_files,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn tasks(&self) -> &Vec<Block> {
        &self.tasks
    }

    pub fn vars(&self) -> &HashMap<String, Variable> {
        &self.vars
    }

    pub fn vars_files(&self) -> &Vec<String> {
        &self.vars_files
    }

    pub fn builder(name: &str, roles: &[Role]) -> PlayBuilder {
        PlayBuilder::new(name, roles)
    }

    pub fn pattern(&self) -> &str {
        self.pattern.as_str()
    }

    pub fn limit(&self) -> Option<&str> {
        self.limit.as_ref().map(|l| l.as_str())
    }

    pub fn tags(&self) -> &Vec<String> {
        &self.tags
    }

    pub fn strategy(&self) -> &Strategy {
        &self.strategy
    }

    pub fn gather_facts(&self) -> Option<bool> {
        self.gather_facts
    }

    pub fn gather_subset(&self) -> &Vec<String> {
        &self.gather_subset
    }

    pub fn gather_timeout(&self) -> u32 {
        self.gather_timeout
    }

    pub fn is_finalized(&self) -> bool {
        self.finalized
    }

    pub fn force_handlers(&self) -> bool {
        self.force_handlers
    }

    pub fn roles(&self) -> &Vec<Role> {
        &self.roles
    }

    fn compile_roles(&self) -> Vec<Block> {
        let mut blocks: Vec<Block> = Vec::new();

        // TODO: compile_rows

        blocks
    }

    pub fn handlers(&self) -> &Vec<Handler> {
        &self.handlers
    }

    pub fn compile(&self) -> Vec<Block> {
        let mut blocks: Vec<Block> = Vec::new();

        // create a block containing a single flush handlers meta
        // task, so we can be sure to run handlers at certain points
        // of the playbook execution
        let mut flush_block = Block::new();

        let meta_task =
            TaskBuilder::new("Flush Handlers", Action::Meta("flush_handlers".to_string()))
                .implicit(true)
                .build();

        if self.tags.is_empty() {
            flush_block.add_to_block(BlockEntry::Task(meta_task));
        } else {
            // TODO: evaluate tags
        };

        if self.force_handlers {
            let noop_task = TaskBuilder::new("NOOP", Action::Meta("noop".to_string()))
                .implicit(true)
                .build();

            // TODO: add remaining blocks
        }

        blocks.extend(self.pre_tasks.clone());
        blocks.push(flush_block.clone());
        blocks.extend(self.compile_roles());
        blocks.extend(self.tasks.clone());
        blocks.push(flush_block.clone());
        blocks.extend(self.post_tasks.clone());
        blocks.push(flush_block);

        blocks
    }
}
