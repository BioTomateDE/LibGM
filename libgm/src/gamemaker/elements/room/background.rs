use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMElement, background::GMBackground},
        reference::GMRef,
        serialize::builder::DataBuilder,
    },
    prelude::*,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Background {
    pub enabled: bool,
    pub foreground: bool,
    pub background_definition: Option<GMRef<GMBackground>>,
    pub x: i32,
    pub y: i32,
    pub tile_x: i32,
    pub tile_y: i32,
    pub speed_x: i32,
    pub speed_y: i32,
    pub stretch: bool,
}

impl GMElement for Background {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let enabled = reader.read_bool32()?;
        let foreground = reader.read_bool32()?;
        let background_definition: Option<GMRef<GMBackground>> =
            reader.read_resource_by_id_opt()?;
        let x = reader.read_i32()?;
        let y = reader.read_i32()?;
        let tile_x = reader.read_i32()?; // Idk if this should be an int instead of a bool
        let tile_y = reader.read_i32()?; // ^
        let speed_x = reader.read_i32()?;
        let speed_y = reader.read_i32()?;
        let stretch = reader.read_bool32()?;

        Ok(Self {
            enabled,
            foreground,
            background_definition,
            x,
            y,
            tile_x,
            tile_y,
            speed_x,
            speed_y,
            stretch,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_bool32(self.enabled);
        builder.write_bool32(self.foreground);
        builder.write_resource_id_opt(self.background_definition);
        builder.write_i32(self.x);
        builder.write_i32(self.y);
        builder.write_i32(self.tile_x);
        builder.write_i32(self.tile_y);
        builder.write_i32(self.speed_x);
        builder.write_i32(self.speed_y);
        builder.write_bool32(self.stretch);
        Ok(())
    }
}
