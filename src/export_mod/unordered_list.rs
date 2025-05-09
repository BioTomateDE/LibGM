use std::any::type_name;
use std::cmp::min;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AModUnorderedListChanges<A> {
    pub additions: Vec<A>,
    pub edits: HashMap<usize, A>,
}

#[derive(Debug, Clone)]
pub struct GModUnorderedListChanges<'o, 'm, G> {
    pub additions: Vec<G>,
    pub edits: HashMap<usize, (&'o G, &'m G)>,
}

pub fn export_changes_unordered_list<'o, 'm, G: PartialEq + Clone>(
    original_data: &'o Vec<G>,
    modified_data: &'m Vec<G>,
) -> Result<GModUnorderedListChanges<'o, 'm, G>, String> {
    let additions: Vec<G> = modified_data.get(original_data.len() .. modified_data.len())
        .ok_or_else(|| format!("Could not get {} additions slice with original data len {} and modified data len {}",
                               type_name::<G>(), original_data.len(), modified_data.len()))?
        .to_vec();

    let mut edits: HashMap<usize, (&G, &G)> = HashMap::new();
    for i in 0..min(original_data.len(), modified_data.len()) {
        if original_data[i] == modified_data[i] {
            continue
        }
        edits.insert(i, (&original_data[i], &modified_data[i]));
    }

    Ok(GModUnorderedListChanges { additions, edits })
}

