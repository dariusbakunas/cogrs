#[derive(Clone, Debug)]
pub struct Host {
    pub name: String,
    groups: Vec<String>,
}

impl Host {
    pub fn new(name: &str) -> Self {
        Host {
            name: name.to_string(),
            groups: Vec::new(),
        }
    }

    pub fn add_group(&mut self, group: &str) {
        let group_name = group.to_string();
        if !self.groups.contains(&group_name) {
            self.groups.push(group_name);
        }
    }

    pub fn populate_ancestors(&mut self, ancestors: Vec<String>) {
        for ancestor_name in &ancestors {
            self.add_group(ancestor_name);
        }
    }
}
