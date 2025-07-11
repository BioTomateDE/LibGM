use std::collections::HashMap;
use crate::gamemaker::deserialize::GMData;
use crate::gamemaker::element::{GMChunkElement, GMElement};
use crate::gamemaker::elements::code::GMVariableType;
use crate::gamemaker::gm_version::GMVersionReq;
use crate::utility::{typename, Stopwatch};


#[derive(Debug, Clone)]
pub struct DataBuilder<'a> {
    pub gm_data: &'a GMData,
    pub raw_data: Vec<u8>,
    /// Keeps track of where padding at the end of chunks starts. Used for undoing the padding for the last chunk.
    pub padding_start_pos: usize,
    /// Pairs data positions of pointer placeholders with the memory address of the GameMaker element they're pointing to
    pub(super) pointer_placeholder_positions: Vec<(u32, usize)>,
    /// Maps memory addresses of GameMaker elements to their resolved data position
    pub(super) pointer_resource_positions: HashMap<usize, u32>,

    pub function_occurrences: Vec<Vec<usize>>,
    pub variable_occurrences: Vec<Vec<(usize, GMVariableType)>>,
}


impl<'a> DataBuilder<'a> {
    pub fn new(gm_data: &'a GMData) -> Self {
        Self {
            gm_data,
            raw_data: Vec::with_capacity(gm_data.original_data_size),
            padding_start_pos: 0,   // should only cause issues if no chunks were written at all (impossible)
            pointer_placeholder_positions: Vec::new(),
            pointer_resource_positions: HashMap::new(),
            function_occurrences: vec![Vec::new(); gm_data.functions.functions.len()],
            variable_occurrences: vec![Vec::new(); gm_data.variables.variables.len()],
        }
    }

    /// The current length (aka. "position") of the internal buffer.
    pub fn len(&self) -> usize {
        self.raw_data.len()
    }

    pub fn is_gm_version_at_least<V: Into<GMVersionReq>>(&self, version_req: V) -> bool {
        self.gm_data.general_info.version.is_version_at_least(version_req)
    }

    pub fn bytecode_version(&self) -> u8 {
        self.gm_data.general_info.bytecode_version
    }

    /// Pads the internal buffer with zero bytes until its length is aligned to `alignment`.
    ///
    /// This adds zero bytes until `self.len()` is a multiple of `alignment`.
    pub fn align(&mut self, alignment: usize) {
        while self.len() % alignment != 0 {
            self.write_u8(0);
        }
    }

    /// Appends the given bytes to the internal data buffer.
    pub fn write_bytes(&mut self, data: &[u8]) {
        self.raw_data.extend_from_slice(data);
    }

    /// Overwrites bytes at the given position in the internal data buffer.
    ///
    /// Useful for patching data like lengths or offsets after serialization.
    fn overwrite_bytes(&mut self, bytes: &[u8], position: usize) -> Result<(), String> {
        if let Some(mut_slice) = self.raw_data.get_mut(position .. position+bytes.len()) {
            mut_slice.copy_from_slice(bytes);
            Ok(())
        } else {
            Err(format!(
                "Could not overwrite {} bytes at position {} in data with length {}; out of bounds",
                bytes.len(), position, self.raw_data.len(),
            ))
        }
    }

    /// Write a GameMaker boolean as a 32-bit integer.
    /// - If `true`, write `1_i32`.
    /// - If `false`, write `0_i32`.
    pub fn write_bool32(&mut self, boolean: bool) {
        self.write_u32(if boolean {1} else {0});
    }

    /// Write an actual character string.
    ///
    /// This should only be used for writing chunk names and in the `STRG` chunk.
    /// For writing regular GameMaker string references, see [`Self::write_gm_string`].
    pub fn write_literal_string(&mut self, string: &str) {
        self.raw_data.extend_from_slice(string.as_bytes());
    }

    /// Write a 4 character ASCII GameMaker chunk name.
    /// Accounts for endianness.
    pub fn write_chunk_name(&mut self, name: &str) -> Result<(), String> {
        if name.len() != 4 {
            return Err(format!(
                "Expected chunk name '{}' to be 4 characters long; but is actually {} characters long",
                name, name.len(),
            ))
        }
        
        let err_fn = || format!(
            "Expected chunk name '{}' to be 4 bytes long; but is actually {} bytes long",
            name, name.as_bytes().len(),
        );

        let bytes: [u8; 4] = if self.gm_data.is_big_endian {
            // reverse the bytes if big endian
            name.bytes().rev().collect::<Vec<u8>>().try_into().map_err(|_| err_fn())?
        } else {
            name.as_bytes().try_into().map_err(|_| err_fn())?
        };
        
        self.write_bytes(&bytes);
        Ok(())
    }

    /// Overwrites a 4-byte unsigned integer (`usize` truncated to `u32`) at `position`.
    ///
    /// Useful for patching fixed-size numeric values like lengths or offsets after serialization.
    /// For writing regular pointer lists, see `write_pointer_list`.
    pub fn overwrite_usize(&mut self, number: usize, position: usize) -> Result<(), String> {
        let number: u32 = number as u32;
        let bytes: [u8; 4] = if self.gm_data.is_big_endian {
            number.to_be_bytes()
        } else {
            number.to_le_bytes()
        };
        self.overwrite_bytes(&bytes, position)
    }

    /// Overwrites a 4-byte signed integer (`i32`) at `position`.
    ///
    /// Useful for patching fixed-size numeric values like lengths or offsets after serialization.
    /// For writing regular pointer lists, see `write_pointer_list`.
    pub fn overwrite_i32(&mut self, number: i32, position: usize) -> Result<(), String> {
        let bytes: [u8; 4] = if self.gm_data.is_big_endian {
            number.to_be_bytes()
        } else {
            number.to_le_bytes()
        };
        self.overwrite_bytes(&bytes, position)
    }

    /// Create a placeholder pointer at the current position in the chunk and remember
    /// its data position paired with the target GameMaker element's memory address.
    ///
    /// This will later be resolved by calling `DataBuilder::resolve_pointer`; replacing the
    /// pointer placeholder with the written data position of the target GameMaker element.
    /// ___
    /// This system exists because it is virtually impossible to predict which data position a GameMaker element will be written to.
    /// Circular references and writing order would make predicting these pointer resource positions even harder.
    /// ___
    /// This function should NOT be called for `GMRef`s; use their `DataBuilder::write_gm_x()` methods instead.
    pub fn write_pointer<T>(&mut self, element: &T) -> Result<(), String> {
        let memory_address: usize = element as *const _ as usize;
        let placeholder_position: u32 = self.len() as u32;  // gamemaker is 32bit anyway
        self.write_u32(0xDEADC0DE);
        self.pointer_placeholder_positions.push((placeholder_position, memory_address));
        Ok(())
    }

    /// Writes a pointer to the given `Option` value.
    /// - If `Some`, writes a pointer to the contained value using `write_pointer`.
    /// - If `None`, writes a null pointer (0) using `write_i32`.
    pub fn write_pointer_opt<T>(&mut self, element: &Option<T>) -> Result<(), String> {
        if let Some(elem) = element {
            self.write_pointer(elem)?;
        } else {
            self.write_i32(0);
        }
        Ok(())
    }

    /// Store the written GameMaker element's data position paired with its memory address in the pointer resource pool.
    /// The element's absolute position corresponds to the data builder's current position,
    /// since this method should get called when the element is serialized.
    pub fn resolve_pointer<T>(&mut self, element: &T) -> Result<(), String> {
        let memory_address: usize = element as *const _ as usize;
        let resource_position: u32 = self.len() as u32;
        if let Some(old_resource_pos) = self.pointer_resource_positions.insert(memory_address, resource_position) {
            return Err(format!(
                "Pointer placeholder for {} with memory address {} already resolved \
                to data position {}; tried to resolve again to data position {}",
                typename::<T>(), memory_address, old_resource_pos, resource_position,
            ))
        }
        Ok(())
    }

    /// Writes a GameMaker data chunk with the given 4-character name.
    /// Skips the chunk if the element does not exist.
    ///
    /// Appends padding if required by the GameMaker version.
    /// This padding has to then be manually cut off for the last chunk in the data file.
    /// # Parameters
    /// - `chunk_name`: 4-character chunk name (e.g., "SCPT", "ROOM").
    /// - `element`: A serializable element implementing `GMElement + GMChunkElement`.
    pub fn build_chunk<T: GMElement+GMChunkElement>(&mut self, chunk_name: &'static str, element: &T) -> Result<(), String> {
        if !element.exists() {
            log::trace!("Skipped building chunk '{chunk_name}' because it does not exist");
            return Ok(())
        }

        let stopwatch = Stopwatch::start();
        self.write_chunk_name(chunk_name)?;
        self.write_u32(0xDEADC0DE);   // chunk length placeholder
        let start_pos: usize = self.len();

        element.serialize(self)
            .map_err(|e| format!("{e}\n↳ while serializing chunk '{chunk_name}'"))?;

        // potentially write padding
        self.padding_start_pos = self.len();
        let ver = &self.gm_data.general_info.version;
        if ver.major >= 2 || (ver.major == 1 && ver.build >= 9999) {
            while self.len() % self.gm_data.chunk_padding != 0 {
                self.write_u8(0);
            }
        }

        let chunk_length: usize = self.len() - start_pos;
        self.overwrite_usize(chunk_length, start_pos - 4)?;   // resolve chunk length placeholder

        log::trace!("Building chunk '{chunk_name}' took {stopwatch}");
        Ok(())
    }
}

