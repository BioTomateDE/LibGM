use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Properties {
    visible: bool,
    anchor: i32,
    stretch_width: bool,
    stretch_height: bool,
    tile_h: bool,
    tile_v: bool,
    keep_aspect_ratio: bool,
}
impl GMElement for Properties {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let visible = reader.read_bool32()?;
        let anchor = reader.read_i32()?;
        let stretch_width = reader.read_bool32()?;
        let stretch_height = reader.read_bool32()?;
        let tile_h = reader.read_bool32()?;
        let tile_v = reader.read_bool32()?;
        let keep_aspect_ratio = reader.read_bool32()?;
        Ok(Self {
            visible,
            anchor,
            stretch_width,
            stretch_height,
            tile_h,
            tile_v,
            keep_aspect_ratio,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_bool32(self.visible);
        builder.write_i32(self.anchor);
        builder.write_bool32(self.stretch_width);
        builder.write_bool32(self.stretch_height);
        builder.write_bool32(self.tile_h);
        builder.write_bool32(self.tile_v);
        builder.write_bool32(self.keep_aspect_ratio);
        Ok(())
    }
}
