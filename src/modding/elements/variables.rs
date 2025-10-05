use serde::{Deserialize, Serialize};
use crate::modding::elements::code::ModInstanceType;
use crate::modding::export::{edit_field_convert, edit_field_option, ModExporter, ModRef};
use crate::gamemaker::elements::variables::{GMVariable, GMVariableB15Data};
use crate::modding::ordered_list::{export_changes_ordered_list, DataChange};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModVariable {
    pub name: ModRef,
    pub instance_type: Option<ModInstanceType>,
}


impl ModExporter<'_, '_> {
    pub fn export_variables(&self) -> Result<Vec<DataChange<ModVariable, ModVariable>>, String> {
        export_changes_ordered_list(
            &self.original_data.variables.variables,
            &self.modified_data.variables.variables,
            |i| self.convert_variable(i),
            |_, m| self.convert_variable(m),
        )
    }
    
    fn convert_variable(&self, i: &GMVariable) -> Result<ModVariable> {
        Ok(ModVariable {
            name: self.convert_string_ref(&i.name)?,
            instance_type: i.b15_data.as_ref().map(|i| self.convert_instance_type(&i.instance_type)).transpose()?,
        })
    }
}

