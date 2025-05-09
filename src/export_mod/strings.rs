use std::collections::HashMap;
use crate::export_mod::export::GModData;
use crate::export_mod::unordered_list::{AModUnorderedListChanges, GModUnorderedListChanges};


impl GModData<'_, '_> {
    pub fn convert_strings(&self, changes: &GModUnorderedListChanges<String>) -> Result<AModUnorderedListChanges<String>, String> {
        let additions: Vec<String> = changes.additions.clone();
        let mut edits: HashMap<usize, String> = HashMap::new();

        for (index, (_original, modified)) in &changes.edits {
            edits.insert(*index, modified.to_string());
        }

        Ok(AModUnorderedListChanges { additions, edits })
    }
}

