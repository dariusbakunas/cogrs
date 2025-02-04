use crate::playbook::play::Play;
use crate::playbook::task::Task;
use crate::vars::variable::Variable;
use std::collections::HashMap;

pub struct VariableManager;

impl VariableManager {
    pub fn new() -> Self {
        Self
    }

    pub fn get_vars(&self) -> HashMap<String, Variable> {
        todo!();
    }

    fn get_magic_vars(
        &self,
        play: Option<&Play>,
        task: Option<&Task>,
        include_hostvars: bool,
    ) -> HashMap<String, Variable> {
        todo!();
    }
}

impl Default for VariableManager {
    fn default() -> Self {
        Self::new()
    }
}
