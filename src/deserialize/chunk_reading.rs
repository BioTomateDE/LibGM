use std::collections::HashMap;
use crate::debug_utils::{format_bytes, typename, unlikely};
use crate::deserialize::functions::GMFunction;
use crate::deserialize::general_info::GMGeneralInfo;
use crate::deserialize::texture_page_items::GMTexturePageItem;
use crate::deserialize::variables::GMVariable;

// GMRef is for parsing chunks:
// It has (fake) generic types to make it clearer
// which type it belongs to (`name: GMRef` vs `name: GMRef<String>`).
// It can be resolved to the data it references using the `.resolve()` method,
// which needs the list the elements are stored in.
// [See GMPointer to understand difference]
#[derive(Hash, PartialEq, Eq)]
pub struct GMRef<T> {
    pub index: u32,
    // marker needs to be here to ignore "unused generic T" error; doesn't store any data
    _marker: std::marker::PhantomData<T>,
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
impl<T> GMRef<T> {
    pub fn new(index: u32) -> GMRef<T> {
        Self {
            index,
            _marker: std::marker::PhantomData,
        }
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


#[derive(Debug, Clone)]
pub struct GMChunk {
    pub name: String,
    pub start_pos: usize,
    pub end_pos: usize,
}

const FAILSAFE_COUNT: u32 = 100_000;    // increase limit is not enough


pub struct GMReader {
    pub general_info: GMGeneralInfo,
    string_occurrence_map: HashMap<usize, GMRef<String>>,
    texture_page_item_occurrence_map: HashMap<usize, GMRef<GMTexturePageItem>>,
    variable_occurrence_map: HashMap<usize, GMRef<GMVariable>>,
    function_occurrence_map: HashMap<usize, GMRef<GMFunction>>,

    data: Vec<u8>,
    chunk: GMChunk,
    cur_pos: usize,
}
impl GMReader {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            general_info: GMGeneralInfo::stub(),
            string_occurrence_map: HashMap::new(),
            texture_page_item_occurrence_map: HashMap::new(),
            variable_occurrence_map: HashMap::new(),
            function_occurrence_map: HashMap::new(),
            chunk: GMChunk {
                name: "grievous_error".to_string(),
                start_pos: 0,
                end_pos: data.len(),
            },
            data,
            cur_pos: 0,
        }
    }

    pub fn read_bytes_dyn(&mut self, count: usize) -> Result<&[u8], String> {
        if self.cur_pos < self.chunk.start_pos {
            return Err(format!(
                "underflowed at reader position {} in chunk '{}': {} < {}",
                self.cur_pos, self.chunk.name, self.cur_pos, self.chunk.start_pos,
            ))
        }
        if self.cur_pos+count > self.chunk.end_pos {
            return Err(format!(
                "overflowed at reader position {} in chunk '{}': {} > {}",
                self.cur_pos, self.chunk.name, self.cur_pos+count, self.chunk.end_pos,
            ))
        }
        // if chunk.start_pos and chunk.end_pos are set correctly; this should never fail
        // it may even be replaced with .unwrap_unchecked() for performance
        let slice: &[u8] = self.data.get(self.cur_pos..self.cur_pos+count).unwrap();
        self.cur_pos += count;
        Ok(slice)
    }
    pub fn read_bytes_const<const N: usize>(&mut self) -> Result<&[u8; N], String> {
        let slice: &[u8] = self.read_bytes_dyn(N)?;
        // read_bytes_dyn is guaranteed to read N bytes so the unwrap never fails.
        // it may even be replaced with .unwrap_unchecked() for performance
        Ok(slice.try_into().unwrap())
    }

    pub fn read_u64(&mut self) -> Result<u64, String> {
        let bytes: &[u8; 8] = self.read_bytes_const().map_err(|e| format!("Trying to read u64 {e}"))?;
        Ok(u64::from_le_bytes(*bytes))
    }
    pub fn read_i64(&mut self) -> Result<i64, String> {
        let bytes: &[u8; 8] = self.read_bytes_const().map_err(|e| format!("Trying to read i64 {e}"))?;
        Ok(i64::from_le_bytes(*bytes))
    }
    pub fn read_u32(&mut self) -> Result<u32, String> {
        let bytes: &[u8; 4] = self.read_bytes_const().map_err(|e| format!("Trying to read u32 {e}"))?;
        Ok(u32::from_le_bytes(*bytes))
    }
    pub fn read_i32(&mut self) -> Result<i32, String> {
        let bytes: &[u8; 4] = self.read_bytes_const().map_err(|e| format!("Trying to read i32 {e}"))?;
        Ok(i32::from_le_bytes(*bytes))
    }
    pub fn read_u16(&mut self) -> Result<u16, String> {
        let bytes: &[u8; 2] = self.read_bytes_const().map_err(|e| format!("Trying to read u16 {e}"))?;
        Ok(u16::from_le_bytes(*bytes))
    }
    pub fn read_i16(&mut self) -> Result<i16, String> {
        let bytes: &[u8; 2] = self.read_bytes_const().map_err(|e| format!("Trying to read i16 {e}"))?;
        Ok(i16::from_le_bytes(*bytes))
    }
    pub fn read_u8(&mut self) -> Result<u8, String> {
        let bytes: &[u8; 1] = self.read_bytes_const().map_err(|e| format!("Trying to read u8 {e}"))?;
        Ok(u8::from_le_bytes(*bytes))
    }
    pub fn read_i8(&mut self) -> Result<i8, String> {
        let bytes: &[u8; 1] = self.read_bytes_const().map_err(|e| format!("Trying to read i8 {e}"))?;
        Ok(i8::from_le_bytes(*bytes))
    }

    pub fn read_f64(&mut self) -> Result<f64, String> {
        let bytes: &[u8; 8] = self.read_bytes_const().map_err(|e| format!("Trying to read f64 {e}"))?;
        Ok(f64::from_le_bytes(*bytes))
    }
    pub fn read_f32(&mut self) -> Result<f32, String> {
        let bytes: &[u8; 4] = self.read_bytes_const().map_err(|e| format!("Trying to read f32 {e}"))?;
        Ok(f32::from_le_bytes(*bytes))
    }

    pub fn read_usize(&mut self) -> Result<usize, String> {
        let number: u32 = self.read_u32()?;
        Ok(number as usize)
    }
    /// Read unsigned 32-bit integer and convert to usize (little endian).
    /// Meant for reading positions/pointers; uses total data length as failsafe.
    /// Automatically subtracts `chunks.abs_pos`; converting it to a relative chunk position.
    pub fn read_pointer(&mut self) -> Result<usize, String> {
        let failsafe_amount: usize = self.data.len();
        let number: usize = self.read_usize()?;
        if number >= failsafe_amount {
            return Err(format!(
                "Failsafe triggered in chunk '{}' at position {} while trying to read usize \
                (pointer) integer: Number {} ({}) is larger than the total data length of {} ({})",
                self.chunk.name, self.cur_pos-4, number, format_bytes(number), failsafe_amount, format_bytes(failsafe_amount),
            ))
        }
        Ok(number)
    }
    
    pub fn read_resource_by_id<T>(&mut self) -> Result<GMRef<T>, String> {
        Ok(GMRef::new(self.read_u32()?))
    }

    pub fn read_resource_by_id_option<T>(&mut self) -> Result<Option<GMRef<T>>, String> {
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

    // /// Read unsigned 32-bit integer and convert to usize (little endian).
    // /// Meant for reading (pointer list element) count; uses small constant number as failsafe.
    // pub fn read_usize_count(&mut self) -> Result<usize, String> {
    //     let number: usize = self.read_usize()?;
    //     if number > FAILSAFE_COUNT {
    //         return Err(format!(
    //             "Failsafe triggered in chunk '{}' at position {} while trying \
    //             to read usize (count) integer: Number {} is larger than the failsafe count of {}",
    //             self.name, self.cur_pos-4, number, FAILSAFE_COUNT,
    //         ))
    //     }
    //     Ok(number)
    // }

    /// Read a 32-bit integer and convert it to a bool.
    /// ___
    /// Returns `Err<String>` when the read number is neither 0 nor 1.
    pub fn read_bool32(&mut self) -> Result<bool, String> {
        let number: u32 = self.read_u32()?;
        match number {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(format!(
                "Read invalid boolean value in chunk '{0}' at position {1}: {2} (0x{2:08X})",
                self.chunk.name, self.cur_pos, number,
            ))
        }
    }

    pub fn read_literal_string(&mut self, length: usize) -> Result<String, String> {
        let bytes: &[u8] = self.read_bytes_dyn(length)
            .map_err(|e| format!("Trying to read literal string with length {length} {e}"))?;
        let string: String = String::from_utf8(bytes.to_vec()).map_err(|e| format!(
            "Could not parse literal string with length {} in chunk '{}' at position {}: {e}",
            length, self.chunk.name, self.cur_pos,
        ))?;
        Ok(string)
    }
    
    /// Read chunk name (4 ascii characters)
    pub fn read_chunk_name(&mut self) -> Result<String, String> {
        if unlikely(self.chunk.start_pos != 0) {
            return Err(format!(
                "Reading a chunk name is only allowed in root; not in a chunk!
                Chunk is called '{}' and has start position {} and end position {}",
                self.chunk.name, self.chunk.start_pos, self.chunk.end_pos,
            ))
        }
        let string: String = self.read_literal_string(4)
            .map_err(|e| if self.cur_pos == 4 {
                "Invalid data.win file; data doesn't start with 'FORM' string".to_string()
            } else {
                format!("Could not parse chunk name at position {}: {e}", self.cur_pos)
            })?;
        if unlikely(string.len() != 4 || !string.is_ascii()) {  // can happen because of unicode
            return Err(format!("Chunk name string \"{string}\" has length {} (chunk names need to be 4 ascii chars long)", string.len()))
        }
        Ok(string)
    }

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

    pub fn read_simple_list<T: GMElement>(&mut self) -> Result<Vec<T>, String> {
        const FAILSAFE_SIZE: usize = 100_000;   // 100 Kilobytes
        let count: usize = self.read_usize()?;
        let implied_data_size: usize = count * size_of::<T>();
        if implied_data_size > FAILSAFE_SIZE {
            return Err(format!(
                "Failsafe triggered in chunk '{}' at position {} while trying \
                to read simple list of {}: Element count {} implies a total data \
                size of {} which is larget than the failsafe size of {}",
                self.chunk.name, self.cur_pos-4, typename::<T>(),
                count, format_bytes(implied_data_size), format_bytes(FAILSAFE_SIZE),
            ))
        }
        let mut elements: Vec<T> = Vec::with_capacity(count);
        for _ in 0..count {
            elements.push(T::deserialize(self)?);
        }
        Ok(elements)
    }

    pub fn read_simple_list_short<T: GMElement>(&mut self) -> Result<Vec<T>, String> {
        const FAILSAFE_SIZE: usize = 1_000;   // 1 Kilobyte
        let count: usize = self.read_u16()? as usize;
        let implied_data_size: usize = count * size_of::<T>();
        if implied_data_size > FAILSAFE_SIZE {
            return Err(format!(
                "Failsafe triggered in chunk '{}' at position {} while trying \
                to read short simple list of {}: Element count {} implies a total data \
                size of {} which is larget than the failsafe size of {}",
                self.chunk.name, self.cur_pos-4, typename::<T>(),
                count, format_bytes(implied_data_size), format_bytes(FAILSAFE_SIZE),
            ))
        }
        let mut elements: Vec<T> = Vec::with_capacity(count);
        for _ in 0..count {
            elements.push(T::deserialize(self)?);
        }
        Ok(elements)
    }

    pub fn read_pointer_list<T: GMElement>(&mut self) -> Result<Vec<T>, String> {
        let pointers: Vec<GMPointer> = self.read_simple_list::<GMPointer>()?;
        let mut elements: Vec<T> = Vec::with_capacity(pointers.len());
        for pointer in pointers {
            self.cur_pos = pointer.pointer_position;
            elements.push(T::deserialize(self)?);
        }
        Ok(elements)
    }

    pub fn read_pointer_list_with_occurrence_map<T: GMElement+GMElementByAbsPos>(&mut self) -> Result<(Vec<T>, HashMap<usize, GMRef<T>>), String> {
        let pointers: Vec<GMPointer> = self.read_simple_list::<GMPointer>()?;
        let mut occurrences: HashMap<usize, GMRef<T>> = HashMap::with_capacity(pointers.len());
        let mut elements: Vec<T> = Vec::with_capacity(pointers.len());
        for (i, pointer) in pointers.iter().enumerate() {
            self.cur_pos = pointer.pointer_position;
            occurrences.insert(self.cur_pos, GMRef::new(i as u32));
            elements.push(T::deserialize(self)?);
        }
        Ok((elements, occurrences))
    }

    pub fn align(&mut self, alignment: usize) -> Result<(), String> {
        while self.cur_pos & (alignment - 1) != 0 {
            self.read_u8()?;
        }
        Ok(())
    }
    
    pub fn set_abs_cur_pos(&mut self, absolute_position: usize) -> Result<(), String> {
        if absolute_position < self.chunk.start_pos || absolute_position > self.chunk.end_pos {
            return Err(format!(
                "Tried to set absolute reader position {} in chunk '{}' with start position {} and end position {}; out of bounds",
                absolute_position, self.chunk.name, self.chunk.start_pos, self.chunk.end_pos,
            ))
        }
        self.cur_pos = absolute_position;
        Ok(())
    }
    pub fn set_rel_cur_pos(&mut self, relative_position: usize) -> Result<(), String> {
        if self.chunk.start_pos + relative_position > self.chunk.end_pos {
            return Err(format!(
                "Tried to set relative reader position to {} in chunk '{}' with start position {} and end position {}; out of bounds",
                relative_position, self.chunk.name, self.chunk.start_pos, self.chunk.end_pos,
            ))
        }
        self.cur_pos = self.chunk.start_pos + relative_position;
        Ok(())
    }
    pub fn skip_bytes(&mut self, bytes_count: usize) {
        self.cur_pos += bytes_count;
    }

    pub fn get_abs_cur_pos(&self) -> usize {
        self.cur_pos
    }
    pub fn get_rel_cur_pos(&self) -> usize {
        self.cur_pos - self.chunk.start_pos
    }
    pub fn get_chunk_length(&self) -> usize {
        self.chunk.end_pos - self.chunk.start_pos
    }

    pub fn assert_chunk_name(&self, chunk_name: &str) -> Result<(), String> {
        if self.chunk.name == chunk_name {
            Ok(())
        } else {
            Err(format!(
                "Expected chunk with name '{}'; got chunk with name '{}' (length: {})",
                self.chunk.name, chunk_name, chunk_name.len(),
            ))
        }
    }
}


fn resolve_occurrence<T>(occurrence_position: usize, occurrence_map: &HashMap<usize, GMRef<T>>, chunk_name: &str, position: usize) -> Result<GMRef<T>, String> {
    occurrence_map.get(&occurrence_position)
        .ok_or_else(|| format!(
            "Could not read {} with absolute position {} in chunk '{}' at position {} \
            because it doesn't exist in the occurrence map (length: {})",
            typename::<T>(), occurrence_position, chunk_name, position, occurrence_map.len(),
        ))
        .cloned()
}

pub trait GMElement {
    fn deserialize(reader: &mut GMReader) -> Result<Self, String> where Self: Sized;
    // fn serialize(builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String>;
}

pub trait GMElementByAbsPos {}
pub trait GMChunkElement {
    fn empty() -> Self;
}


struct GMPointer {
    pointer_position: usize,
}
impl GMElement for GMPointer {
    fn deserialize(reader: &mut GMReader) -> Result<Self, String> {
        let pointer_position: usize = reader.read_usize()?;
        Ok(Self { pointer_position })
    }
    // fn serialize(_: &mut DataBuilder, _: &GMData) -> Result<(), String> {
    //     unimplemented!()
    // }
}


pub fn vec_with_capacity<T>(count: usize) -> Result<Vec<T>, String> {
    const FAILSAFE_SIZE: usize = 1_000_000;   // 1 Megabyte
    let implied_size = size_of::<T>() * count;
    if implied_size > FAILSAFE_SIZE {
        return Err(format!(
            "Failsafe triggered while initializing list of {}: \
            Element count {} implies a total data size of {} which is larget than the failsafe size of {}",
            typename::<T>(), count, format_bytes(implied_size), format_bytes(FAILSAFE_SIZE),
        ))
    }
    Ok(Vec::with_capacity(count))
}

