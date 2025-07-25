use crate::modding::export::ModExporter;
use crate::modding::ordered_list::{export_changes_ordered_list, DataChange};

impl ModExporter<'_, '_> {
    // TODO potential optimization by using references to the data instead of cloning
    pub fn export_audios(&self) -> Result<Vec<DataChange<Vec<u8>, Vec<u8>>>, String> {
        export_changes_ordered_list(
            &self.original_data.audios.audios,
            &self.modified_data.audios.audios,
            |i| Ok(i.audio_data.to_vec()),
            |_, m| Ok(m.audio_data.to_vec()),
        )
    }
}

