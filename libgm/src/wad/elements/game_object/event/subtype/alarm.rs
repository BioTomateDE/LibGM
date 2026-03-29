use crate::{prelude::*, wad::elements::game_object::event::EventSubtype};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Alarm {
    pub index: u32,
}

impl EventSubtype for Alarm {
    fn parse(index: u32) -> Result<Self> {
        if index >= 12 {
            bail!("Alarm index must be less than 12; got {index}");
        }
        Ok(Self { index })
    }

    fn build(self) -> u32 {
        self.index
    }
}
