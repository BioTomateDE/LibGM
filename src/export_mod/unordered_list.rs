use std::cmp::min;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::debug_utils::typename;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditUnorderedList<ADD, EDIT> {
    pub additions: Vec<ADD>,
    pub edits: HashMap<usize, EDIT>,
}


fn export_edits<GM: PartialEq, EDIT>(
    original_list: &[GM],
    modified_list: &[GM],
    map_edit: impl Fn(&GM, &GM) -> Result<EDIT, String>,
) -> Result<HashMap<usize, EDIT>, String> {
    let mut edits: HashMap<usize, EDIT> = HashMap::new();
    for i in 0..min(original_list.len(), modified_list.len()) {
        if original_list[i] == modified_list[i] {
            continue
        }
        edits.insert(i, map_edit(&original_list[i], &modified_list[i])?);
    }
    Ok(edits)
}

pub fn export_changes_unordered_list<GM: PartialEq + Clone, ADD, EDIT>(
    original_list: &[GM],
    modified_list: &[GM],
    map_addition: impl Fn(&GM) -> Result<ADD, String>,
    map_edit: impl Fn(&GM, &GM) -> Result<EDIT, String>,
) -> Result<EditUnorderedList<ADD, EDIT>, String> {
    let additions: Vec<ADD> = modified_list
        .get(original_list.len() .. modified_list.len())
        .ok_or_else(|| format!(
            "Could not get {0} additions slice with original data len {1} and modified data len {2}. \
            If there are purposefully fewer {0}s in your modified data file, please report this as a bug.",
            typename::<GM>(), original_list.len(), modified_list.len(),
        ))?
        .iter()
        .map(map_addition)
        .collect::<Result<Vec<_>, _>>()?;
    
    let edits: HashMap<usize, EDIT> = export_edits(original_list, modified_list, map_edit)?;
    Ok(EditUnorderedList { additions, edits })
}

pub fn export_changes_unordered_list_allow_less<GM: PartialEq + Clone, ADD, EDIT>(
    original_list: &[GM],
    modified_list: &[GM],
    map_addition: impl Fn(&GM) -> Result<ADD, String>,
    map_edit: impl Fn(&GM, &GM) -> Result<EDIT, String>,
) -> Result<EditUnorderedList<ADD, EDIT>, String> { 
    let additions: Vec<ADD> = modified_list
        .get(original_list.len() .. modified_list.len())
        .unwrap_or(&[])
        .iter()
        .map(map_addition)
        .collect::<Result<Vec<_>, _>>()?;

    let edits: HashMap<usize, EDIT> = export_edits(original_list, modified_list, map_edit)?;
    Ok(EditUnorderedList { additions, edits })
}

