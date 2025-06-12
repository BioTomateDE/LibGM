use serde::{Deserialize, Serialize};
use crate::deserialize::sequence::GMSequence;
use crate::export_mod::export::ModExporter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSequence {
    // {~~} {!!} Define fields as needed based on GMSequence
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditSequence {
    // {~~} {!!} Define fields as needed based on GMSequence
}

impl ModExporter<'_, '_> {
    pub fn add_sequence(&self, i: &GMSequence) -> Result<AddSequence, String> {
        todo!()
    }

    pub fn edit_sequence(&self, o: &GMSequence, m: &GMSequence) -> Result<EditSequence, String> {
        todo!()
    }
}

