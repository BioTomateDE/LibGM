use std::any::type_name;
use std::cmp::min;
use std::collections::HashMap;
use serde::Serialize;
use serde_json::json;

pub fn export_changes_unordered_list<G: PartialEq, A: Serialize>(
    original_data: &Vec<G>,
    modified_data: &Vec<G>,
    map_add: fn(&G) -> A,
    map_edit: fn(&G, &G) -> A,
) -> Result<serde_json::Value, String> {
    let additions: Vec<A> = modified_data.get(original_data.len() .. modified_data.len())
        .ok_or_else(|| format!("Could not get {} additions slice with original data len {} and modified data len {}",
                               type_name::<G>(), original_data.len(), modified_data.len()))?
        .iter().map(map_add).collect();

    let mut edits: HashMap<usize, A> = HashMap::new();
    for i in 0..min(original_data.len(), modified_data.len()) {
        if original_data[i] == modified_data[i] {
            continue
        }
        edits.insert(i, map_edit(&original_data[i], &modified_data[i]));
    }

    Ok(json!({
        "add": additions,
        "edit": edits,
    }))
}

