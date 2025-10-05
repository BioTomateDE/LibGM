use serde::{Deserialize, Serialize};
use crate::gamemaker::elements::sequence::GMSequence;
use crate::modding::export::ModExporter;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AddSequence {
    // pub name: ModRef,
    // pub playback: GMSequencePlaybackType,
    // pub playback_speed: f32,
    // pub playback_speed_type: GMAnimSpeedType,
    // pub length: f32,
    // pub origin_x: i32,
    // pub origin_y: i32,
    // pub volume: f32,
    // pub broadcast_messages: Vec<ModRef>,    // String ref
    // pub tracks: Vec<GMTrack>,
    // pub function_ids: HashMap<i32, GMRef<String>>,
    // pub moments: Vec<GMKeyframeMoment>
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditSequence {
    // {~~} {!!} Define fields as needed based on GMSequence
}

impl ModExporter<'_, '_> {
    pub fn add_sequence(&self, i: &GMSequence) -> Result<AddSequence> {
        todo!("sequences not yet implemented")
    }

    pub fn edit_sequence(&self, o: &GMSequence, m: &GMSequence) -> Result<EditSequence> {
        todo!("sequences not yet implemented")
    }
}

