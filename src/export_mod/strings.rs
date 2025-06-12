use crate::export_mod::export::ModExporter;
use crate::export_mod::unordered_list::{export_changes_unordered_list, EditUnorderedList};


impl ModExporter<'_, '_> {
    pub fn export_strings(&self) -> Result<EditUnorderedList<String, String>, String> {
        export_changes_unordered_list(
            &self.original_data.strings.strings_by_index,
            &self.modified_data.strings.strings_by_index,
            |i| Ok(i.clone()),
            |_, m| Ok(m.clone()),
        )
    }
}

