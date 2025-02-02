use std::collections::HashMap;

pub struct VariableManager;

impl VariableManager {
    pub fn new() -> Self {
        Self
    }

    pub fn get_vars(&self) -> HashMap<String, String> {
        todo!();
    }
}

impl Default for VariableManager {
    fn default() -> Self {
        Self::new()
    }
}
