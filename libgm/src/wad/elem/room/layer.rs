// SPDX-License-Identifier: GPL-3.0-only
pub mod assets;
mod background;
pub mod effect;
mod instances;
mod tiles;

pub use self::assets::Assets;
pub use self::background::Background;
pub use self::effect::Effect;
pub use self::instances::Instances;
pub use self::tiles::Tiles;
use crate::gm_enum::gm_enum;
use crate::prelude::*;
use crate::wad::GMVersion;
use crate::wad::elem::GMElement;
use crate::wad::load::reader::DataReader;
use crate::wad::save::writer::DataWriter;

#[derive(Debug, Clone, PartialEq)]
pub struct RoomLayer {
    pub name: GMRef<String>,
    pub id: u32,
    pub ty: Type,
    pub depth: i32,
    pub x_offset: f32,
    pub y_offset: f32,
    pub horizontal_speed: f32,
    pub vertical_speed: f32,
    pub is_visible: bool,
    pub effect_data_2022_1: Option<Data2022_1>,
    pub data: Data,
}

impl GMElement for RoomLayer {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let layer_name: GMRef<String> = reader.read_gm_string()?;
        let layer_id = reader.read_u32()?;
        let layer_type: Type = reader.read_enum()?;
        let layer_depth = reader.read_i32()?;
        let x_offset = reader.read_f32()?;
        let y_offset = reader.read_f32()?;
        let horizontal_speed = reader.read_f32()?;
        let vertical_speed = reader.read_f32()?;
        let is_visible = reader.read_bool32()?;
        let effect_data_2022_1: Option<Data2022_1> =
            reader.deserialize_if_version(GMVersion::GM2022_1)?;

        let data: Data = match layer_type {
            Type::Path | Type::Path2 => Data::None,
            Type::Background => Data::Background(Background::deserialize(reader)?),
            Type::Instances => Data::Instances(Instances::deserialize(reader)?),
            Type::Assets => Data::Assets(Assets::deserialize(reader)?),
            Type::Tiles => Data::Tiles(Tiles::deserialize(reader)?),
            Type::Effect => {
                if reader.version >= GMVersion::GM2022_1 {
                    let effect_data = effect_data_2022_1.as_ref().unwrap();
                    let effect_type = effect_data.effect_type;
                    let properties: Vec<effect::Property> = effect_data.effect_properties.clone();
                    Data::Effect(Effect { effect_type, properties })
                } else {
                    Data::Effect(Effect::deserialize(reader)?)
                }
            }
        };

        Ok(Self {
            name: layer_name,
            id: layer_id,
            ty: layer_type,
            depth: layer_depth,
            x_offset,
            y_offset,
            horizontal_speed,
            vertical_speed,
            is_visible,
            effect_data_2022_1,
            data,
        })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_gm_string(self.name)?;
        writer.write_u32(self.id);
        writer.write_enum(self.ty);
        writer.write_i32(self.depth);
        writer.write_f32(self.x_offset);
        writer.write_f32(self.y_offset);
        writer.write_f32(self.horizontal_speed);
        writer.write_f32(self.vertical_speed);
        writer.write_bool32(self.is_visible);
        writer.write_if_ver(&self.effect_data_2022_1, "Effect Data", GMVersion::GM2022_1)?;
        match &self.data {
            Data::None => {}
            Data::Instances(data) => writer.write_elem(data)?,
            Data::Tiles(data) => writer.write_elem(data)?,
            Data::Background(data) => writer.write_elem(data)?,
            Data::Assets(data) => writer.write_elem(data)?,
            Data::Effect(data) => {
                if writer.version() < GMVersion::GM2022_1 {
                    writer.write_elem(data)?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Data2022_1 {
    pub effect_enabled: bool,
    pub effect_type: GMRef<String>,
    pub effect_properties: Vec<effect::Property>,
}

impl Default for Data2022_1 {
    fn default() -> Self {
        Self {
            effect_enabled: true,
            effect_type: GMRef::none(),
            effect_properties: Vec::new(),
        }
    }
}

impl GMElement for Data2022_1 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let effect_enabled = reader.read_bool32()?;
        let effect_type: GMRef<String> = reader.read_gm_string()?;
        let effect_properties: Vec<effect::Property> = reader.read_simple_list()?;
        Ok(Self {
            effect_enabled,
            effect_type,
            effect_properties,
        })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_bool32(self.effect_enabled);
        writer.write_gm_string(self.effect_type)?;
        writer.write_simple_list(&self.effect_properties)?;
        Ok(())
    }
}

gm_enum!(Type {
    /// unused?
    Path = 0,
    Background = 1,
    Instances = 2,
    Assets = 3,
    Tiles = 4,
    Effect = 6,
    /// introduced in 2024.13
    Path2 = 7,
});

#[derive(Debug, Clone, PartialEq)]
pub enum Data {
    None,
    Instances(Instances),
    Tiles(Tiles),
    Background(Background),
    Assets(Assets),
    Effect(Effect),
}
