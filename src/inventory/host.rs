pub struct Host {
    pub name: String,
}

impl Host {
    pub fn new(name: &str) -> Self {
        Host {
            name: name.to_string(),
        }
    }
}
