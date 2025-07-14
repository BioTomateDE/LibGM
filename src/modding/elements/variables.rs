use serde::{Deserialize, Serialize};
use crate::modding::elements::code::ModInstanceType;
use crate::modding::export::{edit_field, edit_field_convert, wrap_edit_option, EditWrapper, ModExporter, ModRef};
use crate::modding::unordered_list::{export_changes_unordered_list, EditUnorderedList};
use crate::gamemaker::elements::variables::GMVariableB15Data;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddVariable {
    pub name: ModRef,
    pub b15_data: Option<AddVariableB15>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddVariableB15 {
    pub instance_type: ModInstanceType,
    pub variable_id: i32,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditVariable {
    pub name: Option<ModRef>,
    pub b15_data: Option<EditWrapper<AddVariableB15, EditVariableB15>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditVariableB15 {
    pub instance_type: Option<ModInstanceType>,
    pub variable_id: Option<i32>,
}


impl ModExporter<'_, '_> {
    pub fn export_variables(&self) -> Result<EditUnorderedList<AddVariable, EditVariable>, String> {
        export_changes_unordered_list(
            &self.original_data.variables.variables,
            &self.modified_data.variables.variables,
            |i| Ok(AddVariable {
                name: self.convert_string_ref(&i.name)?,
                b15_data: if let Some(ref b15_data) = i.b15_data {Some(self.add_bytecode15_data(b15_data)?)} else {None},
            }),
            |o, m| Ok(EditVariable {
                name: edit_field_convert(&o.name, &m.name, |r| self.convert_string_ref(r))?,
                b15_data: wrap_edit_option(
                    &o.b15_data,
                    &m.b15_data,
                    |i| self.add_bytecode15_data(i),
                    |o, m| self.edit_bytecode15_data(o, m),
                )?,
            }),
            false,
        )
    }
    
    fn add_bytecode15_data(&self, i: &GMVariableB15Data) -> Result<AddVariableB15, String> {
        Ok(AddVariableB15 {
            instance_type: self.convert_instance_type(&i.instance_type)?,
            variable_id: i.variable_id,
        })
    }

    fn edit_bytecode15_data(&self, o: &GMVariableB15Data, m: &GMVariableB15Data) -> Result<EditVariableB15, String> {
        Ok(EditVariableB15 {
            instance_type: edit_field(&o.instance_type, &m.instance_type).map(|i| self.convert_instance_type(&i)).transpose()?,
            variable_id: edit_field(&o.variable_id, &m.variable_id),
        })
    }
}

