use crate::utils::get_unique_id;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Role {
    uuid: String,
    name: String,
    allow_duplicates: bool,
    path: PathBuf,
}

impl Role {
    pub fn new(name: &str, path: &PathBuf) -> Self {
        Self {
            uuid: get_unique_id(false),
            name: name.to_string(),
            allow_duplicates: false,
            path: path.to_path_buf(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn allow_duplicates(&self) -> bool {
        self.allow_duplicates
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn uuid(&self) -> &str {
        &self.uuid
    }
}
