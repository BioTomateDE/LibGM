use std::collections::HashMap;
use crate::deserialize::chunk_reading::GMRef;
use crate::deserialize::strings::GMStrings;


// GMPointer is for building chunks:
// It has to store the kind (data type) of the referenced element,
// because it has to be differentiated from other elements with
// the same index in the pointer pool hashmap.
// Some of them have multiple indexes, because they're contained
// within other elements (events of game objects for example).
// This is important so that their combination of indexes is unique,
// and they can be differentiated in the pool hashmap.
// [See GMRef to understand difference]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GMPointer {
    /// `String`: Used for string references basically everywhere.
    /// Points to actual the actual string, not to the string gm object (which includes the upcoming string length)
    String(usize),
    /// `StringPointerList`: Used for string list in chunk STRG.
    /// Points to the GameMaker object (meaning it points to the string length, not the actual string data).
    /// Effectively `String` minus 4 bytes.
    StringPointerList(usize),
    TexturePage(usize),
    TexturePageData(usize),
    Texture(usize),
    Sprite(usize),
    SpriteSequence(usize),
    SpriteNineSlice(usize),
    Audio(usize),
    Sound(usize),
    Script(usize),
    GameObject(usize),
    GameObjectEvent(usize, usize),
    GameObjectEventInstance(usize, usize, usize),
    GameObjectEventInstanceAction(usize, usize, usize, usize),
    Font(usize),
    FontGlyph(usize, usize),
    Background(usize),
    Path(usize),
    Room(usize),
    RoomBackground(usize, usize),
    RoomBackgroundPointerList(usize),       // pointer to (start of) pointer list
    RoomView(usize, usize),
    RoomViewPointerList(usize),
    RoomGameObject(usize, usize),
    RoomGameObjectPointerList(usize),
    RoomTile(usize, usize),
    RoomTilePointerList(usize),
    RoomLayer(usize, usize),
    RoomLayerPointerList(usize),
    CodeMeta(usize),

    FormLength,
    CodeOccurrence(usize),  // occurrence abs pos
    CodeLength(usize),
}


#[derive(Debug, Clone)]
pub struct DataBuilder {
    pub raw_data: Vec<u8>,
    pub chunk_start_pos: Option<usize>,
    pub pool_placeholders: HashMap<usize, GMPointer>,          // maps gamemaker element references to absolute positions of where they're referenced
    pub placeholder_pool_resources: HashMap<GMPointer, i32>,   // maps gamemaker element references to absolute positions of where their data is OR sometimes any other data
}


impl DataBuilder {
    pub fn start_chunk(&mut self, chunk_name: &str) -> Result<(), String> {
        if let Some(old_chunk_start_pos) = self.chunk_start_pos {
            return Err(format!("Could not start chunk because there is already a chunk being written at position {old_chunk_start_pos}"))
        }
        if chunk_name.len() != 4 {
            return Err(format!("Chunk name '{}' is {} chars long, but needs to be exactly 4 chars long", chunk_name, chunk_name.len()))
        }
        if chunk_name.as_bytes().len() != 4 {   // check char length and byte length to make sure it's a valid ascii string
            return Err(format!("Chunk name '{}' is {} bytes long, but needs to be exactly 4 bytes long", chunk_name, chunk_name.as_bytes().len()))
        }
        self.write_literal_string(chunk_name);
        self.chunk_start_pos = Some(self.len());
        self.write_usize(0xDEAD);
        Ok(())
    }

    pub fn finish_chunk(&mut self) -> Result<(), String> {
        if let Some(chunk_start_pos) = self.chunk_start_pos {
            let chunk_length: usize = self.len() - chunk_start_pos - 4;
            self.overwrite_usize(chunk_length, chunk_start_pos)?;
            self.chunk_start_pos = None;
            Ok(())
        } else {
            Err("Could not finish writing chunk because there is no chunk start position (chunk was never started)".to_string())
        }
    }

    pub fn write_u64(&mut self, number: u64) {
        self.raw_data.extend(number.to_le_bytes());
    }
    pub fn write_i64(&mut self, number: i64) {
        self.raw_data.extend(number.to_le_bytes());
    }
    pub fn write_u32(&mut self, number: u32) {
        self.raw_data.extend(number.to_le_bytes());
    }
    pub fn write_i32(&mut self, number: i32) {
        self.raw_data.extend(number.to_le_bytes());
    }
    pub fn write_u16(&mut self, number: u16) {
        self.raw_data.extend(number.to_le_bytes());
    }
    pub fn write_i16(&mut self, number: i16) {
        self.raw_data.extend(number.to_le_bytes());
    }
    pub fn write_u8(&mut self, number: u8) {
        self.raw_data.extend(number.to_le_bytes());
    }
    pub fn write_i8(&mut self, number: i8) {
        self.raw_data.extend(number.to_le_bytes());
    }
    pub fn write_usize(&mut self, number: usize) {
        self.write_u32(number as u32);
    }
    pub fn write_f64(&mut self, number: f64) {
        self.raw_data.extend(number.to_le_bytes());
    }
    pub fn write_f32(&mut self, number: f32) {
        self.raw_data.extend(number.to_le_bytes());
    }
    pub fn write_bool32(&mut self, boolean: bool) {
        self.write_u32(if boolean {1} else {0});
    }
    pub fn write_literal_string(&mut self, string: &str) {
        self.raw_data.extend_from_slice(string.as_bytes());
    }

    pub fn write_gm_string(&mut self, string_ref: &GMRef<String>) -> Result<(), String> {
        self.write_placeholder(GMPointer::String(string_ref.index))?;
        Ok(())
    }

    /// write a gamemaker string reference to the data if Some else zero
    pub fn write_gm_string_optional(&mut self, string_ref_optional: &Option<GMRef<String>>) -> Result<(), String> {
        match string_ref_optional {
            None => self.write_usize(0),
            Some(string_ref) => self.write_placeholder(GMPointer::String(string_ref.index))?,
        }
        Ok(())
    }
    
    pub fn write_bytes(&mut self, data: &[u8]) {
        self.raw_data.extend_from_slice(data);
    }
    pub fn overwrite_bytes(&mut self, data: &[u8], position: usize) -> Result<(), String> {
        if position + data.len() >= self.len() {
            return Err(format!(
                "Could not overwrite {} bytes at position {} in data with length {} while building chunk with start position {:?}",
                data.len(), position, self.len(), self.chunk_start_pos,
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

    /// write pointer to pointer list (only used in rooms)
    pub fn write_pointer_list(&mut self, pointers: &[usize]) -> Result<(), String> {
        let start_pos_placeholder: usize = self.raw_data.len();
        self.write_usize(0xDEAD);   // will overwrite later

        self.write_usize(pointers.len());

        for pointer in pointers {
            self.write_usize(*pointer);
        }

        self.overwrite_usize(self.raw_data.len(), start_pos_placeholder)?;   // overwrite length placeholder
        Ok(())
    }

    /// Create a placeholder pointer at the current position in the chunk
    /// and store the target gamemaker element (reference) in the pool.
    /// This will later be resolved; replacing the placeholder pointer with
    /// the absolute position of the target data in the data file
    /// (assuming the pointer origin position was added to the pool).
    /// This method should be called when the data file format expects
    /// a pointer to some element, but you don't yet (necessarily) know where
    /// that element will be located in the data file.
    pub fn write_placeholder(&mut self, pointer: GMPointer) -> Result<(), String> {
        let position: usize = self.len();
        self.write_usize(0xDEAD);      // write placeholder
        if let Some(old_value) = self.pool_placeholders.insert(position, pointer.clone()) {
            return Err(format!(
                "Conflicting placeholder positions while pushing placeholder in chunk with start position {:?}: absolute position {} \
                was already set for pointer {:?}; tried to set to new pointer {:?}",
                self.chunk_start_pos, position, old_value, pointer,
            ))
        }
        Ok(())
    }

    /// Store the gamemaker element's absolute position in the pool.
    /// The element's absolute position is the chunk builder's current position;
    /// since this method should get called when the element is built to the data file.
    pub fn resolve_pointer(&mut self, pointer: GMPointer) -> Result<(), String> {
        let position: usize = self.len();
        if let Some(old_value) = self.placeholder_pool_resources.insert(pointer.clone(), position as i32) {
            return Err(format!(
                "Placeholder for {:?} already resolved to absolute position {}; tried to resolve again to position {}",
                pointer, old_value, position,
            ))
        }
        Ok(())
    }

    /// More generic function to write placeholder but with "default value" for the placeholder or whatever
    pub fn write_placeholder_with_data(&mut self, pointer: GMPointer, data: i32) -> Result<(), String> {
        let position: usize = self.len();
        self.write_i32(data);
        if let Some(old_value) = self.pool_placeholders.insert(position, pointer.clone()) {
            return Err(format!(
                "Conflicting placeholder positions while pushing placeholder in chunk with start position {:?}: absolute position {} \
                was already set for pointer {:?}; tried to set to new pointer {:?} with data {}",
                self.chunk_start_pos, position, old_value, pointer, data,
            ))
        }
        Ok(())
    }

    /// More generic function to overwrite placeholder with any data instead of the current position
    pub fn resolve_placeholder(&mut self, pointer: GMPointer, data: i32) -> Result<(), String> {
        if let Some(old_value) = self.placeholder_pool_resources.insert(pointer.clone(), data) {
            return Err(format!(
                "Placeholder for {:?} already resolved to data {}; tried to resolve again to data {}",
                pointer, old_value, data,
            ))
        }
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.raw_data.len()
    }
}


impl GMRef<String> {
    pub fn display<'a>(&self, gm_strings: &'a GMStrings) -> &'a str {
        self.resolve(&gm_strings.strings_by_index)
            .map(|i| i.as_str())
            .unwrap_or("<invalid string reference>")
    }
}

