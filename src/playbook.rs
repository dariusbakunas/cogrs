use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Play {
    pub name: String,
    pub hosts: String,
    pub tasks: Vec<Task>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    pub name: String,
    pub shell: Option<String>,
}

#[allow(dead_code)]
pub fn load_playbook(playbook_path: &PathBuf) -> Vec<Play> {
    let f = std::fs::File::open(playbook_path).expect("Could not open playbook file.");
    let playbook: Vec<Play> = serde_yaml::from_reader(f).expect("Could not parse playbook file.");
    playbook
}
