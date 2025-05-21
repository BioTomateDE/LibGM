use crate::deserialize::chunk_reading::GMRef;
use crate::deserialize::strings::GMStrings;
use crate::serialize::all::DataBuilder;


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Index(pub usize);

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
    String(Index),
    /// `StringPointerList`: Used for string list in chunk STRG.
    /// Points to the GameMaker object (meaning it points to the string length, not the actual string data).
    /// Effectively `String` - 4 bytes.
    StringPointerList(Index),
    TexturePage(Index),
    TexturePageData(Index),
    Texture(Index),
    Sprite(Index),
    SpriteSequence(Index),
    SpriteNineSlice(Index),
    Audio(Index),
    Sound(Index),
    Variable(Index),
    Function(Index),
    Script(Index),
    GameObject(Index),
    GameObjectEvent(Index, Index),
    GameObjectEventInstance(Index, Index, Index),
    GameObjectEventInstanceAction(Index, Index, Index, Index),
    Font(Index),
    FontGlyph(Index, Index),
    Background(Index),
    Path(Index),
    Room(Index),
    RoomBackground(Index, Index),
    RoomView(Index, Index),
    RoomGameObject(Index, Index),
    RoomTile(Index, Index),
    CodeMeta(Index),
    Code(Index),
}
impl GMPointer {
    pub fn string(string_index: usize) -> Self {
        Self::String(Index(string_index))
    }
    pub fn string_pointerlist(string_index: usize) -> Self {
        Self::StringPointerList(Index(string_index))
    }
    pub fn texture_page(texture_page_index: usize) -> Self {
        Self::TexturePage(Index(texture_page_index))
    }
    pub fn texture_page_data(texture_page_data_index: usize) -> Self {
        Self::TexturePageData(Index(texture_page_data_index))
    }
    pub fn texture(texture_index: usize) -> Self {
        Self::Texture(Index(texture_index))
    }
    pub fn sprite(sprite_index: usize) -> Self {
        Self::Sprite(Index(sprite_index))
    }
    pub fn sprite_sequence(sequence_absolute_position: usize) -> Self {
        Self::SpriteSequence(Index(sequence_absolute_position))
    }
    pub fn sprite_nine_slice(nine_slice_absolute_position: usize) -> Self {
        Self::SpriteNineSlice(Index(nine_slice_absolute_position))
    }
    pub fn audio(audio_index: usize) -> Self {
        Self::Audio(Index(audio_index))
    }
    pub fn sound(sound_index: usize) -> Self {
        Self::Sound(Index(sound_index))
    }
    pub fn variable(variable_index: usize) -> Self {
        Self::Variable(Index(variable_index))
    }
    pub fn function(function_index: usize) -> Self {
        Self::Function(Index(function_index))
    }
    pub fn script(script_index: usize) -> Self {
        Self::Script(Index(script_index))
    }
    pub fn game_object(game_object_index: usize) -> Self {
        Self::GameObject(Index(game_object_index))
    }
    pub fn game_object_event(game_object_index: usize, event_index: usize) -> Self {
        Self::GameObjectEvent(Index(game_object_index), Index(event_index))
    }
    pub fn game_object_event_instance(game_object_index: usize, event_index: usize, instance_index: usize) -> Self {
        Self::GameObjectEventInstance(Index(game_object_index), Index(event_index), Index(instance_index))
    }
    pub fn game_object_event_action(game_object_index: usize, event_index: usize, instance_index: usize, action_index: usize) -> Self {
        Self::GameObjectEventInstanceAction(Index(game_object_index), Index(event_index), Index(instance_index), Index(action_index))
    }
    pub fn font(font_index: usize) -> Self {
        Self::Font(Index(font_index))
    }
    pub fn font_glyph(font_index: usize, glyph_index: usize) -> Self {
        Self::FontGlyph(Index(font_index), Index(glyph_index))
    }
    pub fn background(background_index: usize) -> Self {
        Self::Background(Index(background_index))
    }
    pub fn path(path_index: usize) -> Self {
        Self::Path(Index(path_index))
    }
    pub fn room(room_index: usize) -> Self {
        Self::Room(Index(room_index))
    }
    pub fn room_background(room_index: usize, room_background_index: usize) -> Self {
        Self::RoomBackground(Index(room_index), Index(room_background_index))
    }
    pub fn room_view(room_index: usize, view_index: usize) -> Self {
        Self::RoomView(Index(room_index), Index(view_index))
    }
    pub fn room_game_object(room_index: usize, room_game_object_index: usize) -> Self {
        Self::RoomGameObject(Index(room_index), Index(room_game_object_index))
    }
    pub fn room_tile(room_index: usize, tile_index: usize) -> Self {
        Self::RoomTile(Index(room_index), Index(tile_index))
    }
    pub fn code_meta(code_index: usize) -> Self {
        Self::CodeMeta(Index(code_index))
    }
    pub fn code(code_index: usize) -> Self {
        Self::Code(Index(code_index))
    }
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
        data_builder.push_pointer_placeholder(self, GMPointer::string(string_ref.index))?;
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
                "Could not overwrite {} bytes at position {} in data with length {} while building chunk.",
                data.len(), position, self.len(),
            ))
        };

        for (i, byte) in data.iter().enumerate() {
            self.raw_data[position + i] = *byte;
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
