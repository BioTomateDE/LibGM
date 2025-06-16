use crate::export_mod::export::ModExporter;
use crate::export_mod::unordered_list::{export_changes_unordered_list, EditUnorderedList};

impl ModExporter<'_, '_> {
    // TODO potential optimization by using references to the data
    pub fn export_audios(&self) -> Result<EditUnorderedList<Vec<u8>, Vec<u8>>, String> {
        export_changes_unordered_list(
            &self.original_data.audios.audios_by_index,
            &self.modified_data.audios.audios_by_index,
            |i| Ok(i.raw_data.to_vec()),
            |_, m| Ok(m.raw_data.to_vec()),
        )
    }
}

