use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMElement, sequence::GMAnimSpeedType, sprite::GMSprite},
        reference::GMRef,
        serialize::builder::DataBuilder,
    },
    prelude::*,
    util::init::num_enum_from,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Background {
    pub visible: bool,
    pub foreground: bool,
    pub sprite: Option<GMRef<GMSprite>>,
    pub tiled_horizontally: bool,
    pub tiled_vertically: bool,
    pub stretch: bool,
    pub color: u32,
    pub first_frame: f32,
    pub animation_speed: f32,
    pub animation_speed_type: GMAnimSpeedType,
}

impl GMElement for Background {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let visible = reader.read_bool32()?;
        let foreground = reader.read_bool32()?;
        let sprite: Option<GMRef<GMSprite>> = reader.read_resource_by_id_opt()?;
        let tiled_horizontally = reader.read_bool32()?;
        let tiled_vertically = reader.read_bool32()?;
        let stretch = reader.read_bool32()?;
        let color = reader.read_u32()?;
        let first_frame = reader.read_f32()?;
        let animation_speed = reader.read_f32()?;
        let animation_speed_type: GMAnimSpeedType = num_enum_from(reader.read_i32()?)?;

        Ok(Self {
            visible,
            foreground,
            sprite,
            tiled_horizontally,
            tiled_vertically,
            stretch,
            color,
            first_frame,
            animation_speed,
            animation_speed_type,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_bool32(self.visible);
        builder.write_bool32(self.foreground);
        builder.write_resource_id_opt(self.sprite);
        builder.write_bool32(self.tiled_horizontally);
        builder.write_bool32(self.tiled_vertically);
        builder.write_bool32(self.stretch);
        builder.write_u32(self.color);
        builder.write_f32(self.first_frame);
        builder.write_f32(self.animation_speed);
        builder.write_i32(self.animation_speed_type.into());
        Ok(())
    }
}
