//! I literally have no idea what this is.
//! I Copied this from UndertaleModTool in 2025-04-01.
//! Still no idea what YYSWF is.
//! Fuck this module.

mod collision_mask;
pub mod frame;
pub mod item;

pub use collision_mask::CollisionMask;
pub use frame::Frame;
pub use item::Item;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
    util::init::vec_with_capacity,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Data {
    pub swf_version: i32,
    pub yyswf_version: i32,
    pub jpeg_table: Vec<u8>,
    pub timeline: Timeline,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Timeline {
    pub framerate: i32,
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
    pub mask_width: i32,
    pub mask_height: i32,
    pub used_items: Vec<Item>,
    pub frames: Vec<Frame>,
    pub collision_masks: Vec<CollisionMask>,
}

impl GMElement for Timeline {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let used_items: Vec<Item> = reader.read_simple_list()?;
        let framerate = reader.read_i32()?;
        let frames_count = reader.read_u32()?;
        let min_x = reader.read_f32()?;
        let max_x = reader.read_f32()?;
        let min_y = reader.read_f32()?;
        let max_y = reader.read_f32()?;
        let collision_masks_count = reader.read_u32()?;
        let mask_width = reader.read_i32()?;
        let mask_height = reader.read_i32()?;

        let mut frames: Vec<Frame> = vec_with_capacity(frames_count)?;
        for _ in 0..frames_count {
            frames.push(Frame::deserialize(reader)?);
        }

        let mut collision_masks: Vec<CollisionMask> = vec_with_capacity(collision_masks_count)?;
        for _ in 0..collision_masks_count {
            collision_masks.push(CollisionMask::deserialize(reader)?);
        }

        Ok(Self {
            framerate,
            min_x,
            max_x,
            min_y,
            max_y,
            mask_width,
            mask_height,
            used_items,
            frames,
            collision_masks,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_simple_list(&self.used_items)?;
        builder.write_i32(self.framerate);
        builder.write_usize(self.frames.len())?;
        builder.write_f32(self.min_x);
        builder.write_f32(self.max_x);
        builder.write_f32(self.min_y);
        builder.write_f32(self.max_y);
        builder.write_usize(self.collision_masks.len())?;
        builder.write_i32(self.mask_width);
        builder.write_i32(self.mask_height);
        for frame in &self.frames {
            frame.serialize(builder)?;
        }
        for mask in &self.collision_masks {
            mask.serialize(builder)?;
        }
        Ok(())
    }
}

pub const MATRIX33_SIZE: usize = 9;

#[derive(Debug, Clone, PartialEq)]
pub struct Matrix33 {
    pub values: [f32; MATRIX33_SIZE],
}

impl GMElement for Matrix33 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let mut values = [0f32; MATRIX33_SIZE];
        for item in &mut values {
            *item = reader.read_f32()?;
        }
        Ok(Self { values })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        for item in &self.values {
            builder.write_f32(*item);
        }
        Ok(())
    }
}

pub const COLOR_MATRIX_SIZE: usize = 4;

#[derive(Debug, Clone, PartialEq)]
pub struct ColorMatrix {
    pub additive: [i32; COLOR_MATRIX_SIZE],
    pub multiply: [i32; COLOR_MATRIX_SIZE],
}

impl GMElement for ColorMatrix {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let mut additive = [0i32; COLOR_MATRIX_SIZE];
        for item in &mut additive {
            *item = reader.read_i32()?;
        }

        let mut multiply = [0i32; COLOR_MATRIX_SIZE];
        for item in &mut multiply {
            *item = reader.read_i32()?;
        }

        Ok(Self { additive, multiply })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        for item in &self.additive {
            builder.write_i32(*item);
        }
        for item in &self.multiply {
            builder.write_i32(*item);
        }
        Ok(())
    }
}
