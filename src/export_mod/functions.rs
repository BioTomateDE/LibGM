use serde::{Deserialize, Serialize};
use crate::export_mod::export::{edit_field_convert, ModExporter, ModRef};
use crate::export_mod::unordered_list::{export_changes_unordered_list, EditUnorderedList};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddFunction {
    pub name: ModRef,  // String
    pub name_string_id: i32,
    // idk how to handle function and variable occurrences yet
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditFunction {
    pub name: Option<ModRef>,  // String
    pub name_string_id: Option<i32>,
    // idk how to handle function and variable occurrences yet
}


impl ModExporter<'_, '_> {
    pub fn export_functions(&self) -> Result<EditUnorderedList<AddFunction, EditFunction>, String> {
        export_changes_unordered_list(
            &self.original_data.functions.functions_by_index,
            &self.modified_data.functions.functions_by_index,
            |i| Ok(AddFunction {
                name: self.convert_string_ref(i.name)?,
                name_string_id: i.name_string_id,
            }),
            |o, m| Ok(EditFunction {
                name: edit_field_convert(o.name, m.name, |r| self.convert_string_ref(r))?,
                name_string_id: None,
            })
        )
    }
}

