use std::collections::HashMap;
use crate::export_mod::export::GModData;
use crate::export_mod::unordered_list::{EditUnorderedList, GModUnorderedListChanges};


impl GModData<'_, '_> {
    pub fn convert_strings(&self, changes: &GModUnorderedListChanges<String>) -> Result<EditUnorderedList<String, String>, String> {
        let additions: Vec<String> = changes.additions.to_vec();
        let edits: HashMap<usize, String> = changes.edits
            .iter()
            .map(|(i, (_original, modified))| (*i, modified.to_string()))
            .collect();

        Ok(EditUnorderedList { additions, edits })
    }
}

