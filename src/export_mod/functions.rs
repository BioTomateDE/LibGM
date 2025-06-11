use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::deserialize::functions::GMFunction;
use crate::export_mod::export::{edit_field, GModData, ModUnorderedRef};
use crate::export_mod::unordered_list::{EditUnorderedList, GModUnorderedListChanges};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddFunction {
    pub name: ModUnorderedRef,  // String
    // idk how to handle function and variable occurrences yet
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditFunction {
    pub name: Option<ModUnorderedRef>,  // String
    // idk how to handle function and variable occurrences yet
}

impl GModData<'_, '_> {
    fn convert_functions_additions(&self, functions: &[GMFunction]) -> Result<Vec<AddFunction>, String> {
        functions.iter().map(|i| {
            Ok(AddFunction {
                name: self.resolve_string_ref(&i.name)?,
            })
        }).collect()
    }

    pub fn convert_functions(&self, changes: &GModUnorderedListChanges<GMFunction>) -> Result<EditUnorderedList<AddFunction, EditFunction>, String> {
        let additions = self.convert_functions_additions(&changes.additions)?;
        let edits: HashMap<usize, EditFunction> = changes.edits.iter().map(|(i, (o, m))| {
            Ok((*i, EditFunction {
                name: edit_field(&self.resolve_string_ref(&o.name)?, &self.resolve_string_ref(&m.name)?),
            }))
        }).collect::<Result<HashMap<_, _>, String>>()?;
        
        Ok(EditUnorderedList { additions, edits })
    }
}

