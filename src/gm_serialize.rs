use std::collections::HashMap;
use crate::debug_utils::{typename, Stopwatch};
use crate::gamemaker::chunk_reading::{GMChunkElement, GMElement, GMRef};
use crate::gamemaker::texture_page_items::GMTexturePageItem;
use crate::gm_deserialize::GMData;

pub fn build_data_file(gm_data: &GMData) -> Result<Vec<u8>, String> {
    let stopwatch = Stopwatch::start();
    let mut builder = DataBuilder::new(gm_data);
    
    builder.write_literal_string("FORM");
    builder.write_u32(0xDEADC0DE);  // data length placeholder
    
    builder.build_chunk("STRG", &gm_data.strings)?;
    builder.build_chunk("GEN8", &gm_data.general_info)?;
    builder.build_chunk("TXTR", &gm_data.embedded_textures)?;
    builder.build_chunk("TPAG", &gm_data.texture_page_items)?;
    builder.build_chunk("VARI", &gm_data.variables)?;
    builder.build_chunk("FUNC", &gm_data.functions)?;
    builder.build_chunk("SCPT", &gm_data.scripts)?;
    builder.build_chunk("CODE", &gm_data.codes)?;
    builder.build_chunk("FONT", &gm_data.fonts)?;
    builder.build_chunk("SPRT", &gm_data.sprites)?;
    builder.build_chunk("OBJT", &gm_data.game_objects)?;
    builder.build_chunk("ROOM", &gm_data.rooms)?;
    builder.build_chunk("BGND", &gm_data.backgrounds)?;
    builder.build_chunk("PATH", &gm_data.paths)?;
    builder.build_chunk("AUDO", &gm_data.audios)?;
    builder.build_chunk("SOND", &gm_data.sounds)?;
    
    builder.build_chunk("PSYS", &gm_data.particle_systems)?;
    builder.build_chunk("PSEM", &gm_data.particle_emitters)?;
    builder.build_chunk("LANG", &gm_data.language_info)?;
    builder.build_chunk("EXTN", &gm_data.extensions)?;
    builder.build_chunk("AGRP", &gm_data.audio_groups)?;
    builder.build_chunk("GLOB", &gm_data.global_init_scripts)?;
    builder.is_last_chunk = true;
    builder.build_chunk("GMEN", &gm_data.game_end_scripts)?;
    
    builder.overwrite_usize(builder.len() - 8, 4)?;   // overwrite data length placeholder
    log::trace!("Building data file took {stopwatch}");
    Ok(builder.raw_data)
}



#[derive(Debug, Clone)]
pub struct DataBuilder<'a> {
    gm_data: &'a GMData,
    raw_data: Vec<u8>,
    pub is_last_chunk: bool,
    /// Pairs data positions of pointer placeholders with the memory address of the GameMaker element they're pointing to
    pointer_placeholder_positions: Vec<(usize, usize)>,
    /// Maps memory addresses of GameMaker elements to their resolved data position
    pointer_resource_positions: HashMap<usize, usize>,
}


impl<'a> DataBuilder<'a> {
    pub fn new(gm_data: &'a GMData) -> Self {
        Self {
            gm_data,
            raw_data: Vec::new(),
            is_last_chunk: false,
            pointer_placeholder_positions: Vec::new(),
            pointer_resource_positions: HashMap::new(),
        }
    } 
    
    pub fn build_chunk<T: GMElement+GMChunkElement>(&mut self, chunk_name: &str, element: &T) -> Result<(), String> {
        assert_eq!(chunk_name.len(), 4);
        assert_eq!(chunk_name.as_bytes().len(), 4);
        let stopwatch = Stopwatch::start();

        let start_pos: usize = self.len();
        self.write_literal_string(chunk_name);
        self.write_u32(0xDEADC0DE);   // chunk length placeholder
        
        element.serialize(self)
            .map_err(|e| format!("{e}\n>while serializing chunk '{chunk_name}'"))?;
        
        // potentially write padding
        let ver = &self.gm_data.general_info.version;
        if !self.is_last_chunk && ver.major >= 2 || (ver.major == 1 && ver.build >= 9999) {
            while self.len() % self.gm_data.padding != 0 {
                self.write_u8(0);
            }
        }
        
        let chunk_length: usize = self.len() - start_pos;
        if chunk_length == 0 {
            // chunk is completely empty; undo writing chunk name and length placeholder (and padding)
            self.raw_data.truncate(start_pos);
            return Ok(())
        }
        
        self.overwrite_usize(chunk_length, start_pos + 4)?;   // resolve chunk length placeholder
        
        // TODO padding
        log::trace!("Building chunk '{chunk_name}' took {stopwatch}");
        Ok(())
    }

    pub fn write_bytes(&mut self, data: &[u8]) {
        self.raw_data.extend_from_slice(data);
    }
    pub fn write_u64(&mut self, number: u64) {
        self.write_bytes(&number.to_le_bytes());
    }
    pub fn write_i64(&mut self, number: i64) {
        self.write_bytes(&number.to_le_bytes());
    }
    pub fn write_u32(&mut self, number: u32) {
        self.write_bytes(&number.to_le_bytes());
    }
    pub fn write_i32(&mut self, number: i32) {
        self.write_bytes(&number.to_le_bytes());
    }
    pub fn write_u16(&mut self, number: u16) {
        self.write_bytes(&number.to_le_bytes());
    }
    pub fn write_i16(&mut self, number: i16) {
        self.write_bytes(&number.to_le_bytes());
    }
    pub fn write_u8(&mut self, number: u8) {
        self.write_bytes(&number.to_le_bytes());
    }
    pub fn write_i8(&mut self, number: i8) {
        self.write_bytes(&number.to_le_bytes());
    }
    pub fn write_f64(&mut self, number: f64) {
        self.write_bytes(&number.to_le_bytes());
    }
    pub fn write_f32(&mut self, number: f32) {
        self.write_bytes(&number.to_le_bytes());
    }
    pub fn write_usize(&mut self, number: usize) -> Result<(), String> {
        let number: u32 = number.try_into().map_err(|_| format!(
            "Number {number} (0x{number:016X}) does not fit into 32 bits while writing usize integer",
        ))?;
        self.write_u32(number);
        Ok(())
    }
    pub fn write_bool32(&mut self, boolean: bool) {
        self.write_u32(if boolean {1} else {0});
    }
    pub fn write_resource_id<T>(&mut self, resource: &GMRef<T>) {
        self.write_u32(resource.index);
    }
    pub fn write_resource_id_opt<T>(&mut self, resource: &Option<GMRef<T>>) {
        match resource {
            Some(gm_ref) => self.write_u32(gm_ref.index),
            None => self.write_i32(-1),
        }
    }
    pub fn write_literal_string(&mut self, string: &str) {
        self.raw_data.extend_from_slice(string.as_bytes());
    }

    /// Create a placeholder pointer at the current position in the chunk and remember 
    /// its data position paired with the target GameMaker element's memory address.
    /// 
    /// This will later be resolved by calling `DataBuilder::resolve_pointer`; replacing the
    /// pointer placeholder with the written data position of the target GameMaker element.
    /// ___
    /// This system exists because it is virtually impossible to predict which data position a GameMaker element will be written to.
    /// Circular references and writing order would make predicting these pointer resource positions even harder.
    pub fn write_pointer<T: GMElement>(&mut self, element: &T) -> Result<(), String> {
        let memory_address: usize = element as *const _ as usize;
        let placeholder_position: usize = self.len();
        self.write_u32(0xDEADC0DE);
        self.pointer_placeholder_positions.push((placeholder_position, memory_address));
        Ok(())
    }

    /// Store the written GameMaker element's data position paired with its memory address in the pointer resource pool.
    /// The element's absolute position corresponds to the data builder's current position,
    /// since this method should get called when the element is serialized.
    pub fn resolve_pointer<T: GMElement>(&mut self, element: &T) -> Result<(), String> {
        let memory_address: usize = element as *const _ as usize;
        let resource_position: usize = self.len();
        if let Some(old_resource_pos) = self.pointer_resource_positions.insert(memory_address, resource_position) {
            return Err(format!(
                "Pointer placeholder for {} with memory address {} already resolved \
                to data position {}; tried to resolve again to data position {}",
                typename::<T>(), memory_address, old_resource_pos, resource_position,
            ))
        }
        Ok(())
    }

    pub fn write_gm_string(&mut self, gm_string_ref: &GMRef<String>) -> Result<(), String> {
        let resolved_string: &String = gm_string_ref.resolve(&self.gm_data.strings.strings)?;
        // GameMaker string pointers point to the actual character string; not the GameMaker string element
        let memory_address: usize = resolved_string as *const _ as usize;
        let placeholder_position: usize = self.len();
        self.write_u32(0xDEADC0DE);
        self.pointer_placeholder_positions.push((placeholder_position, memory_address));
        Ok(())
    }
    pub fn write_gm_string_opt(&mut self, gm_string_ref_opt: &Option<GMRef<String>>) -> Result<(), String> {
        match gm_string_ref_opt {
            Some(string_ref) => self.write_gm_string(string_ref)?,
            None => self.write_u32(0),
        }
        Ok(())
    }
    pub fn write_gm_texture(&mut self, gm_texture_ref: &GMRef<GMTexturePageItem>) -> Result<(), String> {
        let resolved_texture_page_item: &GMTexturePageItem = gm_texture_ref.resolve(&self.gm_data.texture_page_items.texture_page_items)?;
        self.write_pointer(resolved_texture_page_item)
    }
    pub fn write_gm_texture_opt(&mut self, gm_texture_ref_opt: &Option<GMRef<GMTexturePageItem>>) -> Result<(), String> {
        match gm_texture_ref_opt {
            Some(gm_texture_ref) => self.write_gm_texture(gm_texture_ref)?,
            None => self.write_u32(0),
        }
        Ok(())
    }
    
    pub fn overwrite_bytes(&mut self, data: &[u8], position: usize) -> Result<(), String> {
        if position + data.len() > self.len() {
            return Err(format!(
                "Could not overwrite {} bytes at position {} in data with length {}; out of bounds",
                data.len(), position, self.len(),
            ))
        };
        for (i, byte) in data.iter().enumerate() {
            self.raw_data[position + i] = *byte;
        }
        Ok(())
    }

    pub fn overwrite_usize(&mut self, number: usize, position: usize) -> Result<(), String> {
        let bytes: [u8; 4] = (number as u32).to_le_bytes();
        self.overwrite_bytes(&bytes, position)?;
        Ok(())
    }
    pub fn overwrite_i32(&mut self, number: i32, position: usize) -> Result<(), String> {
        let bytes: [u8; 4] = number.to_le_bytes();
        self.overwrite_bytes(&bytes, position)?;
        Ok(())
    }
    
    pub fn align(&mut self, alignment: usize) {
        while self.len() & (alignment - 1) != 0 {
            self.write_u8(0);
        }
    }

    pub fn len(&self) -> usize {
        self.raw_data.len()
    }
}


