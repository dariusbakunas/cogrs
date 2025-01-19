use std::collections::HashMap;
use serde_yaml::Value;

pub struct Group {
    name: String,
    vars: HashMap<String, Value>,
    hosts: Vec<String>,
    child_groups: Option<Vec<Group>>,
    parent_groups: Option<Vec<Group>>,
}

impl Group {
    pub fn new(name: &str) -> Self {
        Group {
            name: name.to_string(),
            vars: HashMap::new(),
            hosts: Vec::new(),
            child_groups: None,
            parent_groups: None,
        }
    }

    pub fn add_host(&mut self, host_name: &str) {
        let name = host_name.to_string();
        if !self.hosts.contains(&name) {
            self.hosts.push(name);
        }
    }
}
