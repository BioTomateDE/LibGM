use serde::{Deserialize, Serialize};
use crate::deserialize::sequence::GMSequence;
use crate::export_mod::unordered_list::{EditUnorderedList, GModUnorderedListChanges};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSequence {
    // {~~} {!!} Define fields as needed based on GMSequence
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditSequence {
    // {~~} {!!} Define fields as needed based on GMSequence
}

impl GModData<'_, '_> {
    pub fn convert_sequences_additions(&self, gm_sequences: &[GMSequence]) -> Result<Vec<AddSequence>, String> {
        todo!()
    }

    pub fn convert_sequences(&self, changes: GModUnorderedListChanges<GMSequence>) -> Result<EditUnorderedList<AddSequence, EditSequence>, String> {
        todo!()
    }
}

