use std::cmp::Ordering;
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
        if self_index == other_index && matches!(other, DataChange::Delete(_, _)) {
            return Ordering::Greater;
        }

        other_index.cmp(&self_index)
    }
}


#[derive(Debug, Clone)]
struct MyDiffEngine<'a, GM, EDIT, FN>
where
    FN: Fn(&GM) -> Result<EDIT, String>,
{
    changes: Vec<DataChange<EDIT>>,
    modified_data: &'a [GM],
    map_change: FN,
}

impl<'a, GM, EDIT, FN> MyDiffEngine<'a, GM, EDIT, FN>
where
    FN: Fn(&GM) -> Result<EDIT, String>,
{
    fn new(modified_data: &'a [GM], map_change: FN) -> Self {
        Self {
            changes: vec![],
            modified_data,
            map_change,
        }
    }
}

impl<GM: Clone, EDIT, FN> diffs::Diff for MyDiffEngine<'_, GM, EDIT, FN>
where
    FN: Fn(&GM) -> Result<EDIT, String>,
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

    fn insert(&mut self, old: usize, new: usize, new_len: usize) -> Result<(), Self::Error> {
        let data: Vec<EDIT> = self.get_data(new, new_len)?;
        if let Some(DataChange::Insert(last_insertion_index, last_insertion_data)) = self.get_last_change() {
            if old == *last_insertion_index {
                last_insertion_data.extend(data);
                return Ok(())
            }
        }
        self.changes.push(DataChange::Insert(old, data));
        Ok(())
    }

    fn replace(&mut self, old: usize, _old_len: usize, new: usize, new_len: usize) -> Result<(), Self::Error> {
        let data: Vec<EDIT> = self.get_data(new, new_len)?;
        if let Some(DataChange::Edit(_, last_data_edit)) = self.get_last_change() {
            last_data_edit.extend(data);
        } else {
            self.changes.push(DataChange::Edit(old, data))
        }
        Ok(())
    }
}

impl<GM, EDIT, FN> MyDiffEngine<'_, GM, EDIT, FN>
where
    FN: Fn(&GM) -> Result<EDIT, String>,
{
    fn get_last_change(&mut self) -> Option<&mut DataChange<EDIT>> {
        let last_index: usize = match self.changes.len().checked_sub(1) {
            Some(idx) => idx,
            None => return None,
        };
        self.changes.get_mut(last_index)
    }
    
    fn get_data(&self, start: usize, length: usize) -> Result<Vec<EDIT>, String> {
        let end: usize = start + length;
        self.modified_data[start..end].iter().map(&self.map_change).collect::<Result<Vec<_>, String>>()
            .map_err(|e| format!("Error while mapping GameMaker to Acorn elements in ordered list from index {start}-{end}: {e}"))
    }
}


pub fn export_changes_ordered_list<GM: PartialEq + Clone, EDIT, FN>(
    original_data: &[GM],
    modified_data: &[GM],
    map_change: FN,
) -> Result<Vec<DataChange<EDIT>>, String>
where
    FN: Fn(&GM) -> Result<EDIT, String>,
{
    let mut engine: MyDiffEngine<GM, EDIT, FN> = MyDiffEngine::new(modified_data, map_change);

    diffs::myers::diff(&mut engine, original_data, 0, original_data.len(), modified_data, 0, modified_data.len())
        .map_err(|e| format!("Error while generating diffs: {e}"))?;

    Ok(engine.changes)
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
                        .ok_or_else(|| format!("Trying to edit element out of bounds: {} > {}", *index + i, data_list_len))?;
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

