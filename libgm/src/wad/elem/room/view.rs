// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::elem::GMElement;
use crate::wad::elem::game_object::GameObject;
use crate::wad::load::reader::DataReader;
use crate::wad::reference::GMRef;
use crate::wad::save::writer::DataWriter;

/// "Viewports and Cameras"
#[derive(Debug, Clone, PartialEq)]
pub struct RoomView {
    /// "Enable Viewports"
    pub enabled: bool,

    /// "Camera Properties - X Pos"
    pub view_x: i32,

    /// "Camera Properties - Y Pos"
    pub view_y: i32,

    /// "Camera Properties - Width"
    pub view_width: i32,

    /// "Camera Properties - Height"
    pub view_height: i32,

    /// "Viewport Properties - X Pos"
    pub port_x: i32,

    /// "Viewport Properties - Y Pos"
    pub port_y: i32,

    /// "Viewport Properties - Width"
    pub port_width: i32,

    /// "Viewport Properties - Height"
    pub port_height: i32,

    /// "Object Following - Horizontal Border"
    pub border_x: u32,

    /// "Object Following - Vertical Border"
    pub border_y: u32,

    /// "Object Following - Horizontal Speed"
    pub speed_x: i32,

    /// "Object Following - Vertical Speed"
    pub speed_y: i32,

    /// The game object which this camera view follows.
    ///
    /// "Object Following - (Game Object Selector)"
    pub object: GMRef<GameObject>,
}

impl GMElement for RoomView {
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
        let object: GMRef<GameObject> = reader.read_resource_by_id()?;

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

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_bool32(self.enabled);
        writer.write_i32(self.view_x);
        writer.write_i32(self.view_y);
        writer.write_i32(self.view_width);
        writer.write_i32(self.view_height);
        writer.write_i32(self.port_x);
        writer.write_i32(self.port_y);
        writer.write_i32(self.port_width);
        writer.write_i32(self.port_height);
        writer.write_u32(self.border_x);
        writer.write_u32(self.border_y);
        writer.write_i32(self.speed_x);
        writer.write_i32(self.speed_y);
        writer.write_resource_id(self.object);
        Ok(())
    }
}
