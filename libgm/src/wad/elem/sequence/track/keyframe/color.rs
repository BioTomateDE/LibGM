// SPDX-License-Identifier: GPL-3.0-only

use super::Keyframe;
use crate::gm_enum::gm_enum;
use crate::prelude::*;
use crate::wad::elem::GMElement;
use crate::wad::load::reader::DataReader;
use crate::wad::save::writer::DataWriter;

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

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.align(4);
        writer.write_enum(self.interpolation);
        writer.write_simple_list(&self.keyframes)?;
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

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_f32(self.value);
        Ok(())
    }
}

gm_enum!(InterpolationMode {
    None = 0,
    Linear = 1,
});
