use crate::deserialize::all::GMData;
use crate::export_mod::unordered_list::export_changes_unordered_list;

pub fn export_fonts(original_data: &GMData, modified_data: &GMData) -> Result<serde_json::Value, String> {
    export_changes_unordered_list(
        &original_data.strings.strings_by_index,
        &modified_data.strings.strings_by_index,
        |string| string.clone(),
        |_original_string, modified_string| modified_string.clone(),
    )
}

