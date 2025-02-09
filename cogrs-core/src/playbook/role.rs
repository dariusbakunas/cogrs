#[derive(Debug, Clone)]
pub struct Role {
    pub name: String,
}

impl Role {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}
