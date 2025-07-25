use std::cmp::Ordering;
use std::iter::zip;
use serde::{Deserialize, Serialize};
use crate::utility::typename;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataChange<I, E> {
    Insert(usize, Vec<I>),
    Edit(usize, Vec<E>),
    Delete(usize, usize),
}
impl<I, E> DataChange<I, E> {
    fn get_index(&self) -> usize {
        match &self {
            DataChange::Insert(idx, _) => *idx,
            DataChange::Edit(idx, _) => *idx,
            DataChange::Delete(idx, _) => *idx,
        }
    }
}

impl<I, E> PartialEq<Self> for DataChange<I, E> {
    fn eq(&self, _other: &Self) -> bool { unimplemented!() }
}
impl<I, E> Eq for DataChange<I, E> {}
impl<I, E> PartialOrd<Self> for DataChange<I, E> {
    fn partial_cmp(&self, _other: &Self) -> Option<Ordering> { unimplemented!() }
}
impl<I, E> Ord for DataChange<I, E> {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_index: usize = self.get_index();
        let other_index: usize = other.get_index();

        if self_index == other_index && matches!(self, DataChange::Delete(_, _)) {
            return Ordering::Less;
        }
        if self_index == other_index && matches!(other, DataChange::Delete(_, _)) {
            return Ordering::Greater;
        }

        other_index.cmp(&self_index)
    }
}


#[derive(Debug, Clone)]
struct MyDiffEngine<'a, GM, I, E, FI, FE>
where
    FI: Fn(&GM) -> Result<I, String>,
    FE: Fn(&GM, &GM) -> Result<E, String>,
{
    original_data: &'a [GM],
    modified_data: &'a [GM],
    changes: Vec<DataChange<I, E>>,
    map_insert: FI,
    map_edit: FE,
}

impl<'a, GM, I, E, FI, FE> MyDiffEngine<'a, GM, I, E, FI, FE>
where
    FI: Fn(&GM) -> Result<I, String>,
    FE: Fn(&GM, &GM) -> Result<E, String>,
{
    fn new(original_data: &'a [GM], modified_data: &'a [GM], map_edit: FE, map_insert: FI) -> Self {
        Self {
            original_data,
            modified_data,
            changes: vec![],
            map_insert,
            map_edit,
        }
    }
}

impl<GM: Clone, I, E, FI, FE> diffs::Diff for MyDiffEngine<'_, GM, I, E, FI, FE>
where
    FI: Fn(&GM) -> Result<I, String>,
    FE: Fn(&GM, &GM) -> Result<E, String>,
{
    type Error = String;

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

    fn insert(&mut self, old_start: usize, new_start: usize, len: usize) -> Result<(), Self::Error> {
        let mut insertions: Vec<I> = Vec::with_capacity(len);
        for gm_element in &self.modified_data[new_start.. new_start+len] {
            let acorn_element: I = (self.map_insert)(gm_element)
                .map_err(|e| format!("{e}\n↳ while mapping {} insertions in list of {}s", len, typename::<GM>()))?;
            insertions.push(acorn_element);
        }

        if let Some(DataChange::Insert(last_insertion_index, last_insertion_data)) = self.get_last_change() {
            if old_start == *last_insertion_index {
                last_insertion_data.extend(insertions);
                return Ok(())
            }
        }

        self.changes.push(DataChange::Insert(old_start, insertions));
        Ok(())
    }

    fn replace(&mut self, old_start: usize, old_len: usize, new_start: usize, new_len: usize) -> Result<(), Self::Error> {
        if old_len != new_len {
            return Err(format!("Old length is {old_len} but new length is {new_len} while handling myers replacement; report this error"))
        }

        let old_data: &[GM] = &self.original_data[old_start .. old_start+old_len];
        let new_data: &[GM] = &self.modified_data[new_start .. new_start+new_len];

        let mut edits: Vec<E> = Vec::with_capacity(new_len);
        for (old, new) in zip(old_data, new_data) {
            let edit: E = (self.map_edit)(old, new)
                .map_err(|e| format!("{e}\n↳ while mapping {} edits in list of {}s", new_len-new_start, typename::<GM>()))?;
            edits.push(edit);
        }

        if let Some(DataChange::Edit(_, last_data_edit)) = self.get_last_change() {
            last_data_edit.extend(edits);
        } else {
            self.changes.push(DataChange::Edit(old_start, edits))
        }

        Ok(())
    }
}

impl<GM, I, E, FI, FE> MyDiffEngine<'_, GM, I, E, FI, FE>
where
    FI: Fn(&GM) -> Result<I, String>,
    FE: Fn(&GM, &GM) -> Result<E, String>,
{
    fn get_last_change(&mut self) -> Option<&mut DataChange<I, E>> {
        let last_index: usize = match self.changes.len().checked_sub(1) {
            Some(idx) => idx,
            None => return None,
        };
        self.changes.get_mut(last_index)
    }
}


pub fn export_changes_ordered_list<GM: PartialEq + Clone, I, E, FI, FE>(
    original_data: &[GM],
    modified_data: &[GM],
    map_insert: FI,
    map_edit: FE,
) -> Result<Vec<DataChange<I, E>>, String>
where
    FI: Fn(&GM) -> Result<I, String>,
    FE: Fn(&GM, &GM) -> Result<E, String>,
{
    let mut engine: MyDiffEngine<GM, I, E, FI, FE> = MyDiffEngine::new(original_data, modified_data, map_edit, map_insert);

    // {..} slow operation
    diffs::myers::diff(&mut engine, original_data, 0, original_data.len(), modified_data, 0, modified_data.len())
        .map_err(|e| format!("Error while generating diffs: {e}"))?;

    Ok(engine.changes)
}


pub trait DataChanges<I, E> {
    fn flatten(self) -> Vec<DataChange<I, E>>;
}

impl<I, E> DataChanges<I, E> for Vec<DataChange<Option<I>, Option<E>>> {
    fn flatten(self) -> Vec<DataChange<I, E>> {
        self.into_iter().map(|i| match i {
            DataChange::Insert(index, elements) => DataChange::Insert(index, elements.into_iter().flatten().collect()),
            DataChange::Edit(index, elements) => DataChange::Edit(index, elements.into_iter().flatten().collect()),
            DataChange::Delete(index, count) => DataChange::Delete(index, count),
        }).collect()
    }
}


pub fn apply_changes_ordered_list<GM, I, E, FI, FE>(
    data_list: &mut Vec<GM>,
    changes: &mut Vec<DataChange<I, E>>,
    map_insert: FI,
    map_edit: FE,
) -> Result<(), String>
where
    FI: Fn(&I) -> Result<GM, String>,
    FE: Fn(&GM, &E) -> Result<GM, String>,
{
    changes.sort_by(|a, b| a.cmp(b));

    for change in changes {
        match change {
            DataChange::Edit(index, edits) => {
                for (i, edit) in edits.iter().enumerate() {
                    let data_list_len: usize = data_list.len();
                    let element: &mut GM = data_list.get_mut(*index + i)
                        .ok_or_else(|| format!("Trying to edit element out of bounds: {} > {}", *index + i, data_list_len))?;
                    map_edit(element, edit)?;
                }
            }

            DataChange::Insert(index, insertions) => {
                for (i, insertion) in insertions.iter().enumerate() {
                    if *index + i > data_list.len() {
                        return Err(format!("Trying to insert element out of bounds: {} > {}", *index + insertions.len(), data_list.len()))?
                    }
                    data_list.insert(*index + i, map_insert(insertion)?);
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

