use crate::prelude::*;
use crate::gamemaker::deserialize::DataReader;
use crate::gamemaker::serialize::DataBuilder;

pub mod code;
pub mod backgrounds;
pub mod audio_groups;
pub mod data_files;
pub mod animation_curves;
pub mod general_info;
pub mod strings;
pub mod embedded_textures;
pub mod texture_page_items;
pub mod variables;
pub mod scripts;
pub mod functions;
pub mod fonts;
pub mod rooms;
pub mod sequence;
pub mod game_objects;
pub mod embedded_audio;
pub mod sounds;
pub mod sprites;
pub mod sprites_yyswf;
pub mod paths;
pub mod particles;
pub mod options;
pub mod global_init;
pub mod extensions;
pub mod languages;
pub mod shaders;
pub mod ui_nodes;
pub mod timelines;
pub mod embedded_images;
pub mod texture_group_info;
pub mod tags;
pub mod feature_flags;
pub mod filter_effects;


#[allow(unused_variables)]
pub trait GMElement {
    fn deserialize(reader: &mut DataReader) -> Result<Self> where Self: Sized;
    fn serialize(&self, builder: &mut DataBuilder) -> Result<()>;

    fn deserialize_pre_padding(reader: &mut DataReader) -> Result<()> {
        Ok(())
    }
    fn serialize_pre_padding(&self, builder: &mut DataBuilder) -> Result<()> {
        Ok(())
    }
    fn deserialize_post_padding(reader: &mut DataReader, is_last: bool) -> Result<()> {
        Ok(())
    }
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


pub trait GMChunkElement {
    fn stub() -> Self;
    fn exists(&self) -> bool;
}

