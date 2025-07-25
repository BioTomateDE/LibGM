use serde::{Deserialize, Serialize};
use crate::modding::export::{edit_field_convert, ModExporter, ModRef};
use crate::modding::ordered_list::{export_changes_ordered_list, DataChange};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddFunction {
    pub name: ModRef,  // String
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditFunction {
    pub name: Option<ModRef>,  // String
}


impl ModExporter<'_, '_> {
    pub fn export_functions(&self) -> Result<Vec<DataChange<AddFunction, EditFunction>>, String> {
        export_changes_ordered_list(
            &self.original_data.functions.functions,
            &self.modified_data.functions.functions,
            |i| Ok(AddFunction {
                name: self.convert_string_ref(&i.name)?,
            }),
            |o, m| Ok(EditFunction {
                name: edit_field_convert(&o.name, &m.name, |r| self.convert_string_ref(r))?,
            }),
        )
    }
}

