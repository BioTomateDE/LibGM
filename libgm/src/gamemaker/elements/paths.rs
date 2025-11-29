use std::ops::{Deref, DerefMut};

use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMChunkElement, GMElement},
        serialize::builder::DataBuilder,
    },
    prelude::*,
};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GMPaths {
    pub paths: Vec<GMPath>,
    pub exists: bool,
}

impl Deref for GMPaths {
    type Target = Vec<GMPath>;
    fn deref(&self) -> &Self::Target {
        &self.paths
    }
}

impl DerefMut for GMPaths {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.paths
    }
}

impl GMChunkElement for GMPaths {
    const NAME: &'static str = "PATH";
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMPaths {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let paths: Vec<GMPath> = reader.read_pointer_list()?;
        Ok(Self { paths, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_pointer_list(&self.paths)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMPath {
    pub name: String,
    pub is_smooth: bool,
    pub is_closed: bool,
    pub precision: u32,
    pub points: Vec<GMPathPoint>,
}

impl GMElement for GMPath {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: String = reader.read_gm_string()?;
        let is_smooth = reader.read_bool32()?;
        let is_closed = reader.read_bool32()?;
        let precision = reader.read_u32()?;
        let points: Vec<GMPathPoint> = reader.read_simple_list()?;
        Ok(Self {
            name,
            is_smooth,
            is_closed,
            precision,
            points,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.name);
        builder.write_bool32(self.is_smooth);
        builder.write_bool32(self.is_closed);
        builder.write_u32(self.precision);
        builder.write_simple_list(&self.points)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMPathPoint {
    pub x: f32,
    pub y: f32,
    pub speed: f32,
}

impl GMElement for GMPathPoint {
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
