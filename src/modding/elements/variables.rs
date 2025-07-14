use serde::{Deserialize, Serialize};
use crate::modding::elements::code::ModInstanceType;
use crate::modding::export::{edit_field_convert, edit_field_option, ModExporter, ModRef};
use crate::modding::unordered_list::{export_changes_unordered_list, EditUnorderedList};
use crate::gamemaker::elements::variables::GMVariableB15Data;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddVariable {
    pub name: ModRef,
    pub b15_data: Option<ModVariableB15>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditVariable {
    pub name: Option<ModRef>,
    pub b15_data: Option<ModVariableB15>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModVariableB15 {
    pub instance_type: ModInstanceType,
    pub variable_id: i32,
}


impl ModExporter<'_, '_> {
    pub fn export_variables(&self) -> Result<EditUnorderedList<AddVariable, EditVariable>, String> {
        export_changes_unordered_list(
            &self.original_data.variables.variables,
            &self.modified_data.variables.variables,
            |i| Ok(AddVariable {
                name: self.convert_string_ref(&i.name)?,
                b15_data: if let Some(ref b15_data) = i.b15_data {Some(self.convert_bytecode15_data(b15_data)?)} else {None},
            }),
            |o, m| Ok(EditVariable {
                name: edit_field_convert(&o.name, &m.name, |r| self.convert_string_ref(r))?,
                b15_data: edit_field_option(&o.b15_data, &m.b15_data).flatten().map(|i| self.convert_bytecode15_data(&i)).transpose()?,
            }),
            false,
        )
    }
    
    fn convert_bytecode15_data(&self, i: &GMVariableB15Data) -> Result<ModVariableB15, String> {
        Ok(ModVariableB15 {
            instance_type: self.convert_instance_type(&i.instance_type)?,
            variable_id: i.variable_id,
        })
    }
}

