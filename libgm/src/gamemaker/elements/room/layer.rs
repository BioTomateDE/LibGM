pub mod assets;
mod background;
pub mod effect;
mod instances;
mod tiles;

pub use assets::Assets;
pub use background::Background;
pub use effect::Effect;
pub use instances::Instances;
use macros::num_enum;
pub use tiles::Tiles;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::GMElement,
        serialize::{builder::DataBuilder, traits::GMSerializeIfVersion},
    },
    prelude::*,
    util::init::num_enum_from,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Layer {
    pub layer_name: String,
    pub layer_id: u32,
    pub layer_type: Type,
    pub layer_depth: i32,
    pub x_offset: f32,
    pub y_offset: f32,
    pub horizontal_speed: f32,
    pub vertical_speed: f32,
    pub is_visible: bool,
    pub effect_data_2022_1: Option<Data2022_1>,
    pub data: Data,
}

impl GMElement for Layer {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let layer_name: String = reader.read_gm_string()?;
        let layer_id = reader.read_u32()?;
        let layer_type: Type = num_enum_from(reader.read_i32()?)?;
        let layer_depth = reader.read_i32()?;
        let x_offset = reader.read_f32()?;
        let y_offset = reader.read_f32()?;
        let horizontal_speed = reader.read_f32()?;
        let vertical_speed = reader.read_f32()?;
        let is_visible = reader.read_bool32()?;
        let effect_data_2022_1: Option<Data2022_1> = reader.deserialize_if_gm_version((2022, 1))?;

        let data: Data = match layer_type {
            Type::Path | Type::Path2 => Data::None,
            Type::Background => Data::Background(Background::deserialize(reader)?),
            Type::Instances => Data::Instances(Instances::deserialize(reader)?),
            Type::Assets => Data::Assets(Assets::deserialize(reader)?),
            Type::Tiles => Data::Tiles(Tiles::deserialize(reader)?),
            Type::Effect => {
                if reader.general_info.is_version_at_least((2022, 1)) {
                    let effect_data = effect_data_2022_1.as_ref().unwrap();
                    let effect_type = effect_data
                        .effect_type
                        .clone()
                        .ok_or("Effect Type not set for Room Layer 2022.1+ (this error could be a mistake)")?;
                    let properties: Vec<effect::Property> = effect_data.effect_properties.clone();
                    Data::Effect(Effect { effect_type, properties })
                } else {
                    Data::Effect(Effect::deserialize(reader)?)
                }
            },
        };

        Ok(Self {
            layer_name,
            layer_id,
            layer_type,
            layer_depth,
            x_offset,
            y_offset,
            horizontal_speed,
            vertical_speed,
            is_visible,
            effect_data_2022_1,
            data,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.layer_name);
        builder.write_u32(self.layer_id);
        builder.write_i32(self.layer_type.into());
        builder.write_i32(self.layer_depth);
        builder.write_f32(self.x_offset);
        builder.write_f32(self.y_offset);
        builder.write_f32(self.horizontal_speed);
        builder.write_f32(self.vertical_speed);
        builder.write_bool32(self.is_visible);
        self.effect_data_2022_1
            .serialize_if_gm_ver(builder, "Effect Data", (2022, 1))?;
        match &self.data {
            Data::None => {},
            Data::Instances(data) => data.serialize(builder)?,
            Data::Tiles(data) => data.serialize(builder)?,
            Data::Background(data) => data.serialize(builder)?,
            Data::Assets(data) => data.serialize(builder)?,
            Data::Effect(data) => {
                if !builder.is_version_at_least((2022, 1)) {
                    data.serialize(builder)?;
                }
            },
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Data2022_1 {
    pub effect_enabled: bool,
    pub effect_type: Option<String>,
    pub effect_properties: Vec<effect::Property>,
}

impl Default for Data2022_1 {
    fn default() -> Self {
        Self {
            effect_enabled: true,
            effect_type: None,
            effect_properties: Vec::new(),
        }
    }
}

impl GMElement for Data2022_1 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let effect_enabled = reader.read_bool32()?;
        let effect_type: Option<String> = reader.read_gm_string_opt()?;
        let effect_properties: Vec<effect::Property> = reader.read_simple_list()?;
        Ok(Self {
            effect_enabled,
            effect_type,
            effect_properties,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_bool32(self.effect_enabled);
        builder.write_gm_string_opt(&self.effect_type);
        builder.write_simple_list(&self.effect_properties)?;
        Ok(())
    }
}

#[num_enum(i32)]
pub enum Type {
    /// unused?
    Path = 0,
    Background = 1,
    Instances = 2,
    Assets = 3,
    Tiles = 4,
    Effect = 6,
    /// introduced in 2024.13
    Path2 = 7,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Data {
    None,
    Instances(Instances),
    Tiles(Tiles),
    Background(Background),
    Assets(Assets),
    Effect(Effect),
}
