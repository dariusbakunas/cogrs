#[derive(Debug, Clone)]
pub struct Role {
    name: String,
    allow_duplicates: bool,
}

impl Role {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            allow_duplicates: false,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn allow_duplicates(&self) -> bool {
        self.allow_duplicates
    }
}
