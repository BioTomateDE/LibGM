use crate::modding::export::ModExporter;
use crate::modding::unordered_list::{export_changes_unordered_list, EditUnorderedList};


impl ModExporter<'_, '_> {
    pub fn export_strings(&self) -> Result<EditUnorderedList<String, String>, String> {
        export_changes_unordered_list(
            &self.original_data.strings.strings,
            &self.modified_data.strings.strings,
            |i| Ok(i.clone()),
            |_, m| Ok(m.clone()),
            false,
        )
    }
}

