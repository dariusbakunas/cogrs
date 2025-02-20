use play::Play;

pub mod block;
pub mod handler;
pub mod play;
pub mod play_builder;
pub mod play_context;
pub mod role;
pub mod task;

pub struct Playbook {
    file_name: String,
    plays: Vec<Play>,
}

impl Playbook {
    pub fn new(file_name: &str, plays: &[Play]) -> Self {
        Playbook {
            file_name: file_name.to_string(),
            plays: plays.to_vec(),
        }
    }
}
