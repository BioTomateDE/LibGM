use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::deserialize::functions::GMFunction;
use crate::export_mod::export::{edit_field, GModData, ModUnorderedRef};
use crate::export_mod::unordered_list::{ AModUnorderedListChanges, GModUnorderedListChanges};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModFunction {
    pub name: Option<ModUnorderedRef>,  // String
    // idk how to handle function and variable occurrences yet
}

impl GModData<'_, '_> {
    pub fn convert_functions_additions(&self, gm_functions: &Vec<GMFunction>) -> Result<Vec<ModFunction>, String> {
        let mut mod_functions: Vec<ModFunction> = Vec::with_capacity(gm_functions.len());

        for function in gm_functions {
            mod_functions.push(ModFunction {
                name: Some(self.resolve_string_ref(&function.name)?),
            });
        }

        Ok(mod_functions)
    }

    pub fn convert_functions(&self, changes: &GModUnorderedListChanges<GMFunction>) -> Result<AModUnorderedListChanges<ModFunction>, String> {
        let additions: Vec<ModFunction> = self.convert_functions_additions(&changes.additions)?;
        let mut edits: HashMap<usize, ModFunction> = HashMap::new();

        for (index, (original, modified)) in &changes.edits {
            edits.insert(*index, ModFunction {
                name: edit_field(&self.resolve_string_ref(&original.name)?, &self.resolve_string_ref(&modified.name)?),
            });
        }

        Ok(AModUnorderedListChanges { additions, edits })
    }
}

