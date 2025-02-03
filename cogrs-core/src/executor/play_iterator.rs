use crate::playbook::play::Play;

pub struct PlayIterator {
    play: Play,
}

impl PlayIterator {
    pub fn new(play: &Play) -> Self {
        PlayIterator { play: play.clone() }
    }
}
