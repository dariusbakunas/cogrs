#[derive(Clone, Debug)]
pub struct Handler {
    name: String,
    notified_hosts: Vec<String>,
}

impl Handler {
    pub fn new(name: &str) -> Self {
        Handler {
            name: name.to_string(),
            notified_hosts: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_host_notified(&self, name: &str) -> bool {
        self.notified_hosts.contains(&name.to_string())
    }

    pub fn has_notified_hosts(&self) -> bool {
        !self.notified_hosts.is_empty()
    }
}
