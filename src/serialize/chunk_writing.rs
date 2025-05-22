use crate::deserialize::chunk_reading::GMRef;
use crate::deserialize::strings::GMStrings;
use crate::serialize::all::DataBuilder;


// GMPointer is for building chunks:
// It has to store the kind (data type) of the referenced element,
// because it has to be differentiated from other elements with
// the same index in the pointer pool hashmap.
// Some of them have multiple indexes, because they're contained
// within other elements (events of game objects for example).
// This is important so that their combination of indexes is unique
// and they can be differentiated in the pool hashmap.
// [See GMRef to understand difference]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GMPointer {
    /// `String`: Used for string references basically everywhere.
    /// Points to actual the actual string, not to the string gm object (which includes the upcoming string length)
    String(usize),
    /// `StringPointerList`: Used for string list in chunk STRG.
    /// Points to the GameMaker object (meaning it points to the string length, not the actual string data).
    /// Effectively `String` - 4 bytes.
    StringPointerList(usize),
    TexturePage(usize),
    TexturePageData(usize),
    Texture(usize),
    Sprite(usize),
    SpriteSequence(usize),
    SpriteNineSlice(usize),
    Audio(usize),
    Sound(usize),
    Variable(usize),
    Function(usize),
    CodeLocal(usize),
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
    Code(usize),
}


#[derive(Debug, Clone)]
pub struct ChunkBuilder {
    pub raw_data: Vec<u8>,
    pub chunk_name: &'static str,
    pub abs_pos: usize,
}


impl ChunkBuilder {
    pub fn new(data_builder: &mut DataBuilder, name: &'static str) -> Self {
        Self {
            raw_data: vec![],
            chunk_name: name,
            abs_pos: data_builder.len() + 8,
        }
        // abs_pos = data_len+8 to account for chunk name and length which is written before the actual chunk data
    }

    pub fn finish(&self, data_builder: &mut DataBuilder) -> Result<(), String> {
        data_builder.write_chunk_name(self.chunk_name)?;
        data_builder.write_usize(self.raw_data.len());
        data_builder.raw_data.extend(&self.raw_data);
        Ok(())
    }

    pub fn write_u64(&mut self, number: u64) {
        for byte in number.to_le_bytes() {
            self.raw_data.push(byte);
        }
    }
    pub fn write_i64(&mut self, number: i64) {
        for byte in number.to_le_bytes() {
            self.raw_data.push(byte);
        }
    }
    pub fn write_u32(&mut self, number: u32) {
        for byte in number.to_le_bytes() {
            self.raw_data.push(byte);
        }
    }
    pub fn write_i32(&mut self, number: i32) {
        for byte in number.to_le_bytes() {
            self.raw_data.push(byte);
        }
    }
    pub fn write_u16(&mut self, number: u16) {
        for byte in number.to_le_bytes() {
            self.raw_data.push(byte);
        }
    }
    pub fn write_i16(&mut self, number: i16){
        for byte in number.to_le_bytes() {
            self.raw_data.push(byte);
        }
    }
    pub fn write_u8(&mut self, number: u8) {
        for byte in number.to_le_bytes() {
            self.raw_data.push(byte);
        }
    }
    pub fn write_i8(&mut self, number: i8) {
        for byte in number.to_le_bytes() {
            self.raw_data.push(byte);
        }
    }
    pub fn write_usize(&mut self, number: usize) {
        for byte in (number as u32).to_le_bytes() {
            self.raw_data.push(byte);
        }
    }
    pub fn write_f64(&mut self, number: f64) {
        for byte in number.to_le_bytes() {
            self.raw_data.push(byte);
        }
    }
    pub fn write_f32(&mut self, number: f32) {
        for byte in number.to_le_bytes() {
            self.raw_data.push(byte);
        }
    }
    pub fn write_bool32(&mut self, boolean: bool) {
        let number: u32 = if boolean {1} else {0};
        self.write_u32(number);
    }
    pub fn write_literal_string(&mut self, string: &str) -> Result<(), String> {
        // write an utf-8 string to the data
        self.raw_data.extend_from_slice(string.as_bytes());
        Ok(())
    }
    pub fn write_gm_string(&mut self, data_builder: &mut DataBuilder, string_ref: &GMRef<String>) -> Result<(), String> {
        // write a gamemaker string reference to the data
        data_builder.push_pointer_placeholder(self, GMPointer::String(string_ref.index))?;
        Ok(())
    }
    pub fn write_bytes(&mut self, data: &[u8]) {
        for byte in data {
            self.raw_data.push(*byte);
        }
    }
    pub fn overwrite_data(&mut self, data: &[u8], position: usize) -> Result<(), String> {
        if position + data.len() >= self.len() {
            return Err(format!(
                "Could not overwrite {} bytes at position {} in data with length {} while building chunk",
                data.len(), position, self.len(),
            ))
        };

        for (i, byte) in data.iter().enumerate() {
            self.raw_data[position + i] = *byte;
        }

        Ok(())
    }

    /// write pointer to pointer list (only used in rooms)
    pub fn write_pointer_list(&mut self, pointers: &[usize]) -> Result<(), String> {
        let start_pos_placeholder: usize = self.raw_data.len();
        self.write_usize(0); // will overwrite later

        self.write_usize(pointers.len());

        for pointer in pointers {
            let abs_pointer = pointer + self.abs_pos;
            self.write_usize(abs_pointer);
        }

        let pointer_list_start_bytes = (self.raw_data.len() + self.abs_pos).to_le_bytes();
        self.overwrite_data(&pointer_list_start_bytes, start_pos_placeholder)?;
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

