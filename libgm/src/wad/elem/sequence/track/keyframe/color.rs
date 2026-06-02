// SPDX-License-Identifier: GPL-3.0-only

use super::Keyframe;
use crate::gm_enum::gm_enum;
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::GMElement;
use crate::wad::parse::reader::DataReader;

#[derive(Debug, Clone, PartialEq)]
pub struct KeyframesData {
    pub interpolation: InterpolationMode,
    pub keyframes: Vec<Keyframe<Color>>,
}

impl GMElement for KeyframesData {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.align(4)?;
        let interpolation = reader.read_enum()?;
        let keyframes: Vec<Keyframe<Color>> = reader.read_simple_list()?;
        Ok(Self { interpolation, keyframes })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.align(4);
        builder.write_enum(self.interpolation);
        builder.write_simple_list(&self.keyframes)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Color {
    pub value: f32,
}

impl GMElement for Color {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let value = reader.read_f32()?;
        Ok(Self { value })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_f32(self.value);
        Ok(())
    }
}

gm_enum!(InterpolationMode {
    None = 0,
    Linear = 1,
});
