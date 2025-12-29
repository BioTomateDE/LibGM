use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMElement, game_object::GMGameObject},
        reference::GMRef,
        serialize::builder::DataBuilder,
    },
    prelude::*,
};

#[derive(Debug, Clone, PartialEq)]
pub struct View {
    pub enabled: bool,
    pub view_x: i32,
    pub view_y: i32,
    pub view_width: i32,
    pub view_height: i32,
    pub port_x: i32,
    pub port_y: i32,
    pub port_width: i32,
    pub port_height: i32,
    pub border_x: u32,
    pub border_y: u32,
    pub speed_x: i32,
    pub speed_y: i32,
    pub object: Option<GMRef<GMGameObject>>,
}

impl GMElement for View {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let enabled = reader.read_bool32()?;
        let view_x = reader.read_i32()?;
        let view_y = reader.read_i32()?;
        let view_width = reader.read_i32()?;
        let view_height = reader.read_i32()?;
        let port_x = reader.read_i32()?;
        let port_y = reader.read_i32()?;
        let port_width = reader.read_i32()?;
        let port_height = reader.read_i32()?;
        let border_x = reader.read_u32()?;
        let border_y = reader.read_u32()?;
        let speed_x = reader.read_i32()?;
        let speed_y = reader.read_i32()?;
        let object: Option<GMRef<GMGameObject>> = reader.read_resource_by_id_opt()?;

        Ok(Self {
            enabled,
            view_x,
            view_y,
            view_width,
            view_height,
            port_x,
            port_y,
            port_width,
            port_height,
            border_x,
            border_y,
            speed_x,
            speed_y,
            object,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_bool32(self.enabled);
        builder.write_i32(self.view_x);
        builder.write_i32(self.view_y);
        builder.write_i32(self.view_width);
        builder.write_i32(self.view_height);
        builder.write_i32(self.port_x);
        builder.write_i32(self.port_y);
        builder.write_i32(self.port_width);
        builder.write_i32(self.port_height);
        builder.write_u32(self.border_x);
        builder.write_u32(self.border_y);
        builder.write_i32(self.speed_x);
        builder.write_i32(self.speed_y);
        builder.write_resource_id_opt(self.object);
        Ok(())
    }
}
