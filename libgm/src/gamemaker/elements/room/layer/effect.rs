use macros::num_enum;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
    util::init::num_enum_from,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Effect {
    pub effect_type: String,
    pub properties: Vec<Property>,
}

impl GMElement for Effect {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        // {~~} dont serialize_old if >= 2022.1??
        let effect_type: String = reader.read_gm_string()?;
        let properties: Vec<Property> = reader.read_simple_list()?;
        Ok(Self { effect_type, properties })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.effect_type);
        builder.write_simple_list(&self.properties)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Property {
    pub kind: PropertyKind,
    pub name: String,
    pub value: String,
}

impl GMElement for Property {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let kind: PropertyKind = num_enum_from(reader.read_i32()?)?;
        let name: String = reader.read_gm_string()?;
        let value: String = reader.read_gm_string()?;
        Ok(Self { kind, name, value })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(self.kind.into());
        builder.write_gm_string(&self.name);
        builder.write_gm_string(&self.value);
        Ok(())
    }
}

#[num_enum(i32)]
pub enum PropertyKind {
    Real = 0,
    Color = 1,
    Sampler = 2,
}
