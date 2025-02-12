use play::Play;

pub mod block;
pub mod play;
pub mod role;
pub mod task;

pub struct Playbook {
    file_name: String,
    plays: Vec<Play>,
}

impl Playbook {
    pub fn new(file_name: String, plays: &[Play]) -> Self {
        Playbook {
            file_name,
            plays: plays.to_vec(),
        }
    }
}
