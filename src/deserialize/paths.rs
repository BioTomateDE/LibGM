use crate::deserialize::chunk_reading::{GMChunkElement, GMElement, DataReader, GMRef};


#[derive(Debug, Clone)]
pub struct GMPaths {
    pub paths: Vec<GMPath>,
    pub exists: bool,
}
impl GMChunkElement for GMPaths {
    fn empty() -> Self {
        Self { paths: vec![], exists: false }
    }
}
impl GMElement for GMPaths {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let paths: Vec<GMPath> = reader.read_pointer_list::<GMPath>()?;
        Ok(GMPaths { paths, exists: true })
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMPath {
    pub name: GMRef<String>,
    pub is_smooth: bool,
    pub is_closed: bool,
    pub precision: u32,
    pub points: Vec<GMPathPoint>,
}
impl GMElement for GMPath {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let is_smooth: bool = reader.read_bool32()?;
        let is_closed: bool = reader.read_bool32()?;
        let precision: u32 = reader.read_u32()?;
        let points: Vec<GMPathPoint> = reader.read_simple_list()?;
        Ok(GMPath { name, is_smooth, is_closed, precision, points })
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMPathPoint {
    pub x: f32,
    pub y: f32,
    pub speed: f32,
}
impl GMElement for GMPathPoint {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let x: f32 = reader.read_f32()?;
        let y: f32 = reader.read_f32()?;
        let speed: f32 = reader.read_f32()?;
        Ok(GMPathPoint { x, y, speed })
    }
}

