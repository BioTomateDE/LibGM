use serde::{Deserialize, Serialize};
use crate::modding::export::{ModExporter, ModRef};
use crate::modding::unordered_list::{export_changes_unordered_list, EditUnorderedList};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModScript {
    pub name: ModRef,
    pub is_constructor: bool,
    pub code: Option<ModRef>,
}


impl ModExporter<'_, '_> {
    pub fn export_scripts(&self) -> Result<EditUnorderedList<ModScript, ModScript>, String> {
        export_changes_unordered_list(
            &self.original_data.scripts.scripts,
            &self.modified_data.scripts.scripts,
            |i| Ok(ModScript {
                name: self.convert_string_ref(&i.name)?,
                is_constructor: i.is_constructor,
                code: self.convert_code_ref_opt(&i.code)?,
            }),
            |_, m| Ok(ModScript {
                name: self.convert_string_ref(&m.name)?,
                is_constructor: m.is_constructor,
                code: self.convert_code_ref_opt(&m.code)?,
            }),
            false,
        )
    }
}

