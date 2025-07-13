use rayon::iter::{IntoParallelIterator, ParallelIterator, IntoParallelRefIterator};
use std::cmp::min;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::utility::typename;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditUnorderedList<ADD, EDIT> {
    pub additions: Vec<ADD>,
    pub edits: HashMap<usize, EDIT>,
}


fn export_edits<GM: PartialEq + Sync, EDIT: Send>(
    original_list: &[GM],
    modified_list: &[GM],
    map_edit: impl Fn(&GM, &GM) -> Result<EDIT, String> + Sync,
) -> Result<HashMap<usize, EDIT>, String> {
    let min_len: usize = min(original_list.len(), modified_list.len());
    let results: Vec<(usize, Result<EDIT, String>)> = (0..min_len)
        .into_par_iter()
        .filter_map(|i| {
            if original_list[i] != modified_list[i] {
                Some((i, map_edit(&original_list[i], &modified_list[i])))
            } else {
                None
            }
        })
        .collect();

    let mut edits: HashMap<usize, EDIT> = HashMap::new();
    for (i, result) in results {
        edits.insert(i, result?);    // propagate first error encountered
    }
    Ok(edits)
}


pub fn export_changes_unordered_list<GM, ADD, EDIT>(
    original_list: &[GM],
    modified_list: &[GM],
    map_addition: impl Fn(&GM) -> Result<ADD, String> + Send + Sync,
    map_edit: impl Fn(&GM, &GM) -> Result<EDIT, String> + Send + Sync,
    // TODO remove this and instead apply export_changes_ordered_list for lists where elements can be removed
    allow_length_mismatch: bool,    // don't throw error if there are fewer elements in the modified list than in the original list
) -> Result<EditUnorderedList<ADD, EDIT>, String> where
    GM: PartialEq + Clone + Send + Sync,
    ADD: Send, EDIT: Send
{
    // this range should correspond to the added elements; assuming none were removed and new elems were pushed to the end of the list
    let range = original_list.len() .. modified_list.len();
    let additions: Vec<ADD> = if allow_length_mismatch {
        modified_list.get(range).unwrap_or(&[])
    } else {
        modified_list.get(range).ok_or_else(|| format!(
            "Could not get {0} additions slice with original data len {1} and modified data len {2}. \
            If there are purposefully fewer {0}s in your modified data file, please report this as a bug.",
            typename::<GM>(), original_list.len(), modified_list.len(),
        ))?
    }.par_iter().map(map_addition).collect::<Result<Vec<_>, _>>()?;
    // TODO: does this par_iter implicitly send the entire ModExporter struct across threads?
    
    let edits: HashMap<usize, EDIT> = export_edits(original_list, modified_list, map_edit)?;
    Ok(EditUnorderedList { additions, edits })
}

