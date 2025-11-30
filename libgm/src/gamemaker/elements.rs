use crate::{
    gamemaker::{deserialize::reader::DataReader, serialize::builder::DataBuilder},
    prelude::*,
};

pub mod animation_curves;
pub mod audio_groups;
pub mod backgrounds;
pub mod code;
pub(crate) mod data_files;
pub mod embedded_audio;
pub mod embedded_images;
pub mod embedded_textures;
pub mod extensions;
pub mod feature_flags;
pub mod filter_effects;
pub mod fonts;
pub mod functions;
pub mod game_end;
pub mod game_objects;
pub mod general_info;
pub mod global_init;
pub mod languages;
pub mod options;
pub mod particle_emitters;
pub mod particle_systems;
pub mod paths;
pub mod rooms;
pub mod scripts;
pub mod sequence;
pub mod shaders;
pub mod sounds;
pub mod sprites;
pub mod sprites_yyswf;
pub(crate) mod strings;
pub mod tags;
pub mod texture_group_info;
pub mod texture_page_items;
pub mod timelines;
pub mod ui_nodes;
pub mod variables;

#[allow(unused_variables)]
/// All GameMaker elements that can be deserialized
/// from a data file should implement this trait.
pub(crate) trait GMElement: Sized {
    /// Deserializes this element from the current position of the reader.
    ///
    /// Implementations should read the exact binary representation of this element
    /// and return a fully constructed instance.
    fn deserialize(reader: &mut DataReader) -> Result<Self>;

    /// Serializes this element to the current position of the builder.
    ///
    /// Implementations should write the exact binary representation of this element
    /// in the format expected by the GameMaker runtime.
    fn serialize(&self, builder: &mut DataBuilder) -> Result<()>;

    /// Handles padding bytes that may appear before this element in pointer lists.
    ///
    /// This is called before [`GMElement::deserialize`] when reading from structured data.
    /// The default implementation does nothing - override if your element requires
    /// alignment padding in specific contexts.
    fn deserialize_pre_padding(reader: &mut DataReader) -> Result<()> {
        Ok(())
    }

    /// Writes padding bytes that may be required before this element in pointer lists.
    ///
    /// This is called before [`GMElement::serialize`] when writing to structured data.
    /// The default implementation does nothing - override if your element requires
    /// alignment padding in specific contexts.
    fn serialize_pre_padding(&self, builder: &mut DataBuilder) -> Result<()> {
        Ok(())
    }

    /// Handles padding bytes that may appear after this element in pointer lists.
    ///
    /// This is called after [`GMElement::deserialize`] when reading from structured data.
    /// The `is_last` parameter indicates if this is the final element in a list,
    /// which may affect padding requirements.
    fn deserialize_post_padding(reader: &mut DataReader, is_last: bool) -> Result<()> {
        Ok(())
    }

    /// Writes padding bytes that may be required after this element in pointer lists.
    ///
    /// This is called after [`GMElement::serialize`] when writing to structured data.
    /// The `is_last` parameter indicates if this is the final element in a list,
    /// which may affect padding requirements.
    fn serialize_post_padding(&self, builder: &mut DataBuilder, is_last: bool) -> Result<()> {
        Ok(())
    }
}

impl GMElement for u8 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_u8()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_u8(*self);
        Ok(())
    }
}

impl GMElement for i8 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_i8()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i8(*self);
        Ok(())
    }
}

impl GMElement for u16 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_u16()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_u16(*self);
        Ok(())
    }
}

impl GMElement for i16 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_i16()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i16(*self);
        Ok(())
    }
}

impl GMElement for u32 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_u32()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_u32(*self);
        Ok(())
    }
}

impl GMElement for i32 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_i32()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(*self);
        Ok(())
    }
}

impl GMElement for u64 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_u64()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_u64(*self);
        Ok(())
    }
}

impl GMElement for i64 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_i64()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i64(*self);
        Ok(())
    }
}

impl GMElement for f32 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_f32()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_f32(*self);
        Ok(())
    }
}

impl GMElement for f64 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_f64()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_f64(*self);
        Ok(())
    }
}

impl GMElement for bool {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_bool32()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_bool32(*self);
        Ok(())
    }
}

/// All chunk elements should implement this trait.
pub(crate) trait GMChunkElement: GMElement + Default {
    /// The four character GameMaker chunk name (GEN8, STRG, VARI, etc.).
    const NAME: &'static str;

    /// Returns `true` if this chunk is present in the data file.
    ///
    /// This differs from simply checking if the chunk is empty:
    /// - A list chunk may exist and contain zero elements.
    ///   > Chunk name + chunk length (four) + element count (zero).
    /// - A chunk may exist but contain no data.
    ///   > Chunk name + chunk length (zero).
    /// - A chunk may be absent entirely from the file format.
    ///   > Completely gone.
    ///
    /// Use this to distinguish between "present but empty" and "not present at all".
    fn exists(&self) -> bool;
}
