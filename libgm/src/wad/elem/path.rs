// SPDX-License-Identifier: GPL-3.0-only

use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::gm_named_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::parse::reader::DataReader;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Paths {
    pub elems: Vec<Option<Path>>,
    pub exists: bool,
}

gm_named_list_chunk!(PATH, Paths, Path, nullable);

impl GMElement for Paths {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let elems: Vec<Option<Path>> = reader.read_pointer_list_opt()?;
        Ok(Self { elems, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_pointer_list_opt(&self.elems)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    pub name: GMRef<String>,
    pub is_smooth: bool,
    pub is_closed: bool,
    pub precision: u32,
    pub points: Vec<Point>,
}

impl GMElement for Path {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let is_smooth = reader.read_bool32()?;
        let is_closed = reader.read_bool32()?;
        let precision = reader.read_u32()?;
        let points: Vec<Point> = reader.read_simple_list()?;
        Ok(Self {
            name,
            is_smooth,
            is_closed,
            precision,
            points,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(self.name)?;
        builder.write_bool32(self.is_smooth);
        builder.write_bool32(self.is_closed);
        builder.write_u32(self.precision);
        builder.write_simple_list(&self.points)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub speed: f32,
}

impl GMElement for Point {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let x = reader.read_f32()?;
        let y = reader.read_f32()?;
        let speed = reader.read_f32()?;
        Ok(Self { x, y, speed })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_f32(self.x);
        builder.write_f32(self.y);
        builder.write_f32(self.speed);
        Ok(())
    }
}
