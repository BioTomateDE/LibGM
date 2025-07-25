use crate::modding::export::ModExporter;
use crate::modding::ordered_list::{export_changes_ordered_list, DataChange};

impl ModExporter<'_, '_> {
    pub fn export_strings(&self) -> Result<Vec<DataChange<String, String>>, String> {
        export_changes_ordered_list(
            &self.original_data.strings.strings,
            &self.modified_data.strings.strings,
            |i| Ok(i.clone()),
            |_, m| Ok(m.clone()),
        )
    }
}

