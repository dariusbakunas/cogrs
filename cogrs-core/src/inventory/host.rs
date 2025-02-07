use crate::vars::variable::Variable;
use indexmap::IndexMap;

#[derive(Clone, Debug)]
pub struct Host {
    pub name: String,
    implicit: bool,
    vars: IndexMap<String, Variable>,
    groups: Vec<String>,
}

impl Host {
    pub fn new(name: &str) -> Self {
        Host {
            name: name.to_string(),
            groups: Vec::new(),
            vars: IndexMap::new(),
            implicit: false,
        }
    }

    pub fn get_vars(&self) -> &IndexMap<String, Variable> {
        &self.vars
    }

    pub fn set_vars(&mut self, vars: IndexMap<String, Variable>) {
        self.vars = vars;
    }

    pub fn set_var(&mut self, key: &str, value: &Variable) {
        self.vars.insert(key.to_string(), value.clone());
    }

    pub fn is_implicit(&self) -> bool {
        self.implicit
    }

    pub fn add_group(&mut self, group: &str) {
        let group_name = group.to_string();
        if !self.groups.contains(&group_name) {
            self.groups.push(group_name);
        }
    }

    pub fn get_groups(&self) -> &Vec<String> {
        &self.groups
    }

    pub fn populate_ancestors(&mut self, ancestors: Vec<String>) {
        for ancestor_name in &ancestors {
            self.add_group(ancestor_name);
        }
    }
}
