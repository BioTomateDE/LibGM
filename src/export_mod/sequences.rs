use serde::{Deserialize, Serialize};
use crate::deserialize::sequence::GMSequence;
use crate::export_mod::export::GModData;
use crate::export_mod::unordered_list::{AModUnorderedListChanges, GModUnorderedListChanges};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModSequence {
    // {~~} {!!} Define fields as needed based on GMSequence
}

impl GModData<'_, '_> {
    pub fn convert_sequences_additions(&self, gm_sequences: Vec<GMSequence>) -> Result<ModSequence, String> {
        todo!()
    }

    pub fn convert_sequences(&self, changes: GModUnorderedListChanges<GMSequence>) -> Result<AModUnorderedListChanges<ModSequence>, String> {
        todo!()
    }
}

