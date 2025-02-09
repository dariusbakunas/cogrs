use crate::playbook::block::Block;
use crate::playbook::play::Play;

pub struct PlayIterator<'a> {
    blocks: Vec<Block>,
    play: &'a Play,
}

impl<'a> PlayIterator<'a> {
    pub fn new(play: &'a Play) -> Self {
        let blocks = Vec::new();

        // TODO: create setup block

        PlayIterator { blocks, play }
    }
}
