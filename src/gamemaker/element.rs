use crate::gamemaker::deserialize::DataReader;
use crate::gamemaker::serialize::DataBuilder;

#[allow(unused_variables)]
pub trait GMElement {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> where Self: Sized;
    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String>;

    fn deserialize_pre_padding(reader: &mut DataReader) -> Result<(), String> {
        Ok(())
    }
    fn serialize_pre_padding(&self, builder: &mut DataBuilder) -> Result<(), String> {
        Ok(())
    }
    fn deserialize_post_padding(reader: &mut DataReader, is_last: bool) -> Result<(), String> {
        Ok(())
    }
    fn serialize_post_padding(&self, builder: &mut DataBuilder, is_last: bool) -> Result<(), String> {
        Ok(())
    }
}

impl GMElement for u8 {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        reader.read_u8()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_u8(*self);
        Ok(())
    }
}
impl GMElement for i8 {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        reader.read_i8()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_i8(*self);
        Ok(())
    }
}
impl GMElement for u16 {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        reader.read_u16()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_u16(*self);
        Ok(())
    }
}
impl GMElement for i16 {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        reader.read_i16()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_i16(*self);
        Ok(())
    }
}
impl GMElement for u32 {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        reader.read_u32()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_u32(*self);
        Ok(())
    }
}
impl GMElement for i32 {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        reader.read_i32()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_i32(*self);
        Ok(())
    }
}
impl GMElement for u64 {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        reader.read_u64()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_u64(*self);
        Ok(())
    }
}
impl GMElement for i64 {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        reader.read_i64()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_i64(*self);
        Ok(())
    }
}
impl GMElement for f32 {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        reader.read_f32()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_f32(*self);
        Ok(())
    }
}
impl GMElement for f64 {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        reader.read_f64()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_f64(*self);
        Ok(())
    }
}
impl GMElement for usize {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        reader.read_usize()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_usize(*self)?;
        Ok(())
    }
}
impl GMElement for bool {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        reader.read_bool32()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_bool32(*self);
        Ok(())
    }
}


pub trait GMChunkElement {
    fn stub() -> Self;
    fn exists(&self) -> bool;
}

