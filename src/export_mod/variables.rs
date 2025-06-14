use serde::{Deserialize, Serialize};
use crate::export_mod::code::ModInstanceType;
use crate::export_mod::export::{edit_field, edit_field_convert, ModExporter, ModRef};
use crate::export_mod::unordered_list::{export_changes_unordered_list, EditUnorderedList};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddVariable {
    pub name: ModRef,
    pub instance_type: ModInstanceType,
    pub variable_id: Option<i32>,
    pub name_string_id: i32,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditVariable {
    pub name: Option<ModRef>,
    pub instance_type: Option<ModInstanceType>,
    pub variable_id: Option<Option<i32>>,
    pub name_string_id: Option<i32>,
}


impl ModExporter<'_, '_> {
    pub fn export_variables(&self) -> Result<EditUnorderedList<AddVariable, EditVariable>, String> {
        export_changes_unordered_list(
            &self.original_data.variables.variables,
            &self.modified_data.variables.variables,
            |i| Ok(AddVariable {
                name: self.convert_string_ref(i.name)?,
                instance_type: self.convert_instance_type(&i.instance_type)?,
                variable_id: i.variable_id,
                name_string_id: i.name_string_id,
            }),
            |o, m| Ok(EditVariable {
                name: edit_field_convert(o.name, m.name, |r| self.convert_string_ref(r))?,
                instance_type: edit_field(&self.convert_instance_type(&o.instance_type)?, &self.convert_instance_type(&m.instance_type)?),
                variable_id: edit_field(&o.variable_id, &m.variable_id),
                name_string_id: edit_field(&o.name_string_id, &m.name_string_id),
            }),
        )
    }
}

