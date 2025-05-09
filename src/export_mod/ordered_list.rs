use std::cmp::Ordering;
use std::iter::Map;
use std::slice::Iter;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataChange<T> {
    Edit(usize, Vec<T>),
    Insert(usize, Vec<T>),
    Delete(usize, usize),
}
impl<T> DataChange<T> {
    fn get_index(&self) -> usize {
        match &self {
            DataChange::Edit(idx, _) => *idx,
            DataChange::Insert(idx, _) => *idx,
            DataChange::Delete(idx, _) => *idx,
        }
    }
}

impl<T> Eq for DataChange<T> {}
impl<T> PartialEq<Self> for DataChange<T> {
    fn eq(&self, _other: &Self) -> bool { unimplemented!() }
}
impl<T> PartialOrd<Self> for DataChange<T> {
    fn partial_cmp(&self, _other: &Self) -> Option<Ordering> { unimplemented!() }
}
impl<T> Ord for DataChange<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_index: usize = self.get_index();
        let other_index: usize = other.get_index();

        if self_index == other_index && matches!(self, DataChange::Delete(_, _)) {
            return Ordering::Less;
        }
        if  self_index == other_index && matches!(other, DataChange::Delete(_, _)) {
            return Ordering::Greater;
        }

        other_index.cmp(&self_index)
    }
}

// G stands for GameMaker data
// A stands for AcornGM data
#[derive(Debug, Clone)]
struct MyDiffEngine<'a, G, A> {
    changes: Vec<DataChange<A>>,
    modified_data: &'a Vec<G>,
    map_change: fn(&G) -> A,
}

impl<'a, G, A> MyDiffEngine<'a, G, A> {
    fn new(modified_data: &'a Vec<G>, map_change: fn(&G) -> A) -> Self {
        Self {
            changes: vec![],
            modified_data,
            map_change,
        }
    }
}

impl<G: Clone, A> diffs::Diff for MyDiffEngine<'_, G, A> {
    type Error = std::convert::Infallible;

    fn delete(&mut self, old: usize, len: usize, _new: usize) -> Result<(), Self::Error> {
        if let Some(DataChange::Delete(last_deletion_index, last_deletion_length)) = self.get_last_change() {
            if old == *last_deletion_index {
                *last_deletion_length += len;
                return Ok(())
            }
        }
        self.changes.push(DataChange::Delete(old, len));
        Ok(())
    }

    fn insert(&mut self, old: usize, new: usize, new_len: usize) -> Result<(), Self::Error> {
        let data: Map<Iter<G>, fn(&G) -> A> = self.modified_data[new..new+new_len].iter().map(self.map_change);
        if let Some(DataChange::Insert(last_insertion_index, last_insertion_data)) = self.get_last_change() {
            if old == *last_insertion_index {
                last_insertion_data.extend(data);
                return Ok(())
            }
        }
        self.changes.push(DataChange::Insert(old, data.collect()));
        Ok(())
    }

    fn replace(&mut self, old: usize, _old_len: usize, new: usize, new_len: usize) -> Result<(), Self::Error> {
        let data: Map<Iter<G>, fn(&G) -> A> = self.modified_data[new..new+new_len].iter().map(self.map_change);
        if let Some(DataChange::Edit(_, last_editment_data)) = self.get_last_change() {
            last_editment_data.extend(data);
        } else {
            self.changes.push(DataChange::Edit(old, data.collect()))
        }
        Ok(())
    }
}

impl<G, A> MyDiffEngine<'_, G, A> {
    fn get_last_change(&mut self) -> Option<&mut DataChange<A>> {
        let last_index: usize = match self.changes.len().checked_sub(1) {
            Some(idx) => idx,
            None => return None,
        };
        self.changes.get_mut(last_index)
    }
}


pub fn export_changes_ordered_list<G: PartialEq + Clone, A>(original_data: &Vec<G>, modified_data: &Vec<G>, map_change: fn(&G) -> A) -> Vec<DataChange<A>> {
    let mut engine: MyDiffEngine<G, A> = MyDiffEngine::new(modified_data, map_change);

    diffs::myers::diff(&mut engine, original_data, 0, original_data.len(), modified_data, 0, modified_data.len())
        .map_err(|e| format!("Error while generating diffs: {e}")).unwrap();  // REMOVE .unwrap IF NOT Infallible

    engine.changes
}


pub fn apply_changes_ordered_list<G, A>(
    data_list: &mut Vec<G>,
    changes: &mut Vec<DataChange<A>>,
    map_insert: fn(&A) -> G,
    map_edit: fn(&mut G, &A),
) -> Result<(), String> {
    changes.sort_by(|a, b| a.cmp(b));

    for change in changes {
        match change {
            DataChange::Edit(index, edits) => {
                for (i, edit) in edits.iter().enumerate() {
                    let data_list_len: usize = data_list.len();
                    let element: &mut G = data_list.get_mut(*index + i)
                        .ok_or(format!("Trying to edit element out of bounds: {} > {}", *index + i, data_list_len))?;
                    map_edit(element, edit);
                }
            }

            DataChange::Insert(index, insertions) => {
                for (i, insertion) in insertions.iter().enumerate() {
                    if *index + i > data_list.len() {
                        return Err(format!("Trying to insert element out of bounds: {} > {}", *index + insertions.len(), data_list.len()))?
                    }
                    data_list.insert(*index + i, map_insert(insertion));
                }
            }

            DataChange::Delete(index, length) => {
                if *index + *length > data_list.len() {
                    return Err(format!("Trying to delete elements out of bounds: {} > {}", *index + *length, data_list.len()))?
                }
                data_list.drain(*index .. *index + *length);
            }
        }
    }
    Ok(())
}

