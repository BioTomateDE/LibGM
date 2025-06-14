use std::collections::HashMap;
use crate::deserialize::chunk_reading::{GMChunk, GMRef};
use crate::deserialize::strings::GMStrings;

#[derive(Debug, Clone, PartialEq)]
pub struct GMPath {
    pub name: GMRef<String>,
    pub is_smooth: bool,
    pub is_closed: bool,
    pub precision: u32,
    pub points: Vec<GMPathPoint>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMPathPoint {
    pub x: f32,
    pub y: f32,
    pub speed: f32,
}

#[derive(Debug, Clone)]
pub struct GMPaths {
    pub paths_by_index: Vec<GMPath>,  // paths by index/order in chunk PATH
}

pub fn parse_chunk_path(chunk: &mut GMChunk, strings: &GMStrings) -> Result<GMPaths, String> {
    chunk.cur_pos = 0;
    let path_count: usize = chunk.read_usize_count()?;
    let mut path_starting_positions: Vec<usize> = Vec::with_capacity(path_count);
    for _ in 0..path_count {
        let start_position: usize = chunk.read_usize_pos()? - chunk.abs_pos;
        path_starting_positions.push(start_position);
    }

    let mut paths_by_index: Vec<GMPath> = Vec::with_capacity(path_count);
    let mut abs_pos_to_ref: HashMap<usize, GMRef<GMPath>> = HashMap::with_capacity(path_count);
    for (i, start_position) in path_starting_positions.iter().enumerate() {
        chunk.cur_pos = *start_position;

        let name: GMRef<String> = chunk.read_gm_string(&strings)?;
        let is_smooth: bool = chunk.read_bool32()?;
        let is_closed: bool = chunk.read_bool32()?;
        let precision: u32 = chunk.read_u32()?;
        let points: Vec<GMPathPoint> = parse_path_points(chunk)?;

        let path: GMPath = GMPath {
            name,
            is_smooth,
            is_closed,
            precision,
            points,
        };
        abs_pos_to_ref.insert(start_position + chunk.abs_pos, GMRef::new(i));
        paths_by_index.push(path);
    }
    Ok(GMPaths { paths_by_index})
}


fn parse_path_points(chunk: &mut GMChunk) -> Result<Vec<GMPathPoint>, String> {
    let point_count: usize = chunk.read_usize_count()?;
    let mut points: Vec<GMPathPoint> = Vec::with_capacity(point_count);

    for _ in 0..point_count {
        let x: f32 = chunk.read_f32()?;
        let y: f32 = chunk.read_f32()?;
        let speed: f32 = chunk.read_f32()?;
        points.push(GMPathPoint {
            x,
            y,
            speed,
        })
    }

    Ok(points)
}

