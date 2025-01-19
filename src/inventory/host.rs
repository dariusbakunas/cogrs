pub struct Host {
    name: String,
}

impl Host {
    pub fn new(name: &str) -> Self {
        Host {
            name: name.to_string(),
        }
    }
}