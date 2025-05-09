use std::collections::HashMap;
use crate::deserialize::fonts::{GMFont, GMFontGlyph};
use crate::export_mod::export::{edit_field, edit_field_option, GModData, ModUnorderedRef};
use crate::export_mod::unordered_list::{export_changes_unordered_list, AModUnorderedListChanges, GModUnorderedListChanges};


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

