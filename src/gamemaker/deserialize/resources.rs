use std::collections::HashMap;
use crate::gamemaker::deserialize::reader::DataReader;
use crate::gamemaker::elements::texture_page_items::GMTexturePageItem;
use crate::utility::typename;


/// GMRef has (fake) generic types to make it clearer which type it belongs to (`name: GMRef` vs `name: GMRef<String>`).
/// It can be resolved to the data it references using the `.resolve()` method, which needs the list the elements are stored in.
/// This means that removing or inserting elements in the middle of the list will shift all their `GMRef`s; breaking them.
#[derive(Hash, PartialEq, Eq)]
pub struct GMRef<T> {
    pub index: u32,
    // marker needs to be here to ignore "unused generic T" error; doesn't store any data
    _marker: std::marker::PhantomData<T>,
}

impl<T> GMRef<T> {
    pub fn new(index: u32) -> GMRef<T> {
        Self {
            index,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T> Clone for GMRef<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for GMRef<T> {}
impl<T> std::fmt::Debug for GMRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GMRef<{}#{}>", typename::<T>(), self.index)
    }
}

impl<'a, T> GMRef<T> {
    pub fn resolve(&self, elements_by_index: &'a Vec<T>) -> Result<&'a T, String> {
        elements_by_index.get(self.index as usize)
            .ok_or_else(|| format!(
                "Could not resolve {} reference with index {} in list with length {}",
                std::any::type_name::<T>(),
                self.index,
                elements_by_index.len(),
            ))
    }
}


impl DataReader<'_> {
    pub fn read_gm_string(&mut self) -> Result<GMRef<String>, String> {
        let occurrence_position: usize = self.read_usize()?;
        resolve_occurrence(occurrence_position, &self.string_occurrence_map, &self.chunk.name, self.cur_pos)
    }

    pub fn read_gm_texture(&mut self) -> Result<GMRef<GMTexturePageItem>, String> {
        let occurrence_position: usize = self.read_usize()?;
        resolve_occurrence(occurrence_position, &self.texture_page_item_occurrence_map, &self.chunk.name, self.cur_pos)
    }

    pub fn read_gm_string_opt(&mut self) -> Result<Option<GMRef<String>>, String> {
        let occurrence_position: usize = self.read_usize()?;
        if occurrence_position == 0 {
            return Ok(None)
        }
        Ok(Some(resolve_occurrence(occurrence_position, &self.string_occurrence_map, &self.chunk.name, self.cur_pos)?))
    }

    pub fn read_gm_texture_opt(&mut self) -> Result<Option<GMRef<GMTexturePageItem>>, String> {
        let occurrence_position: usize = self.read_usize()?;
        if occurrence_position == 0 {
            return Ok(None)
        }
        Ok(Some(resolve_occurrence(occurrence_position, &self.texture_page_item_occurrence_map, &self.chunk.name, self.cur_pos)?))
    }

    pub fn resolve_gm_str(&self, string_ref: GMRef<String>) -> Result<&String, String> {
        string_ref.resolve(&self.strings.strings)
    }

    pub fn display_gm_str(&self, string_ref: GMRef<String>) -> &str {
        string_ref.display(&self.strings)
    }

    pub fn read_resource_by_id<T>(&mut self) -> Result<GMRef<T>, String> {
        Ok(GMRef::new(self.read_u32()?))
    }

    pub fn read_resource_by_id_opt<T>(&mut self) -> Result<Option<GMRef<T>>, String> {
        // TODO: either remove failsafe or also implement it in `read_resource_by_id`
        const FAILSAFE_COUNT: u32 = 100_000;    // increase limit is not enough
        let number: i32 = self.read_i32()?;
        if number == -1 {
            return Ok(None)
        }
        let number: u32 = number.try_into().map_err(|_| format!(
            "Invalid negative number {number} (0x{number:08X}) while reading optional resource by ID",
        ))?;
        if number > FAILSAFE_COUNT {
            return Err(format!(
                "Failsafe triggered in chunk '{}' at position {} \
                while reading optional resource by ID: \
                Number {} is larger than the failsafe count of {}",
                self.chunk.name, self.cur_pos - 4, number, FAILSAFE_COUNT,
            ))
        }
        Ok(Some(GMRef::new(number)))
    }
}


fn resolve_occurrence<T>(occurrence_position: usize, occurrence_map: &HashMap<usize, GMRef<T>>, chunk_name: &str, position: usize) -> Result<GMRef<T>, String> {
    match occurrence_map.get(&occurrence_position) {
        Some(gm_ref) => Ok(gm_ref.clone()),
        None => Err(format!(
            "Could not read {} with absolute position {} in chunk '{}' at position {} \
            because it doesn't exist in the occurrence map (length: {})",
            typename::<T>(), occurrence_position, chunk_name, position, occurrence_map.len(),
        ))
    }
}

