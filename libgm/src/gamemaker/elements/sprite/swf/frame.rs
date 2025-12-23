use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{
            GMElement,
            sprite::swf::{ColorMatrix, Matrix33},
        },
        serialize::builder::DataBuilder,
    },
    prelude::*,
    util::init::vec_with_capacity,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub frame_objects: Vec<Object>,
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}

impl GMElement for Frame {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let frame_object_count = reader.read_u32()?;
        let min_x = reader.read_f32()?;
        let max_x = reader.read_f32()?;
        let min_y = reader.read_f32()?;
        let max_y = reader.read_f32()?;
        let mut frame_objects: Vec<Object> = vec_with_capacity(frame_object_count)?;
        for _ in 0..frame_object_count {
            frame_objects.push(Object::deserialize(reader)?);
        }
        Ok(Self {
            frame_objects,
            min_x,
            max_x,
            min_y,
            max_y,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_usize(self.frame_objects.len())?;
        builder.write_f32(self.min_x);
        builder.write_f32(self.max_x);
        builder.write_f32(self.min_y);
        builder.write_f32(self.max_y);
        for frame_object in &self.frame_objects {
            frame_object.serialize(builder)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    pub char_id: i32,
    pub char_index: i32,
    pub depth: i32,
    pub clipping_depth: i32,
    pub transformation_matrix: Matrix33,
    pub color_matrix: ColorMatrix,
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}

impl GMElement for Object {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let char_id = reader.read_i32()?;
        let char_index = reader.read_i32()?;
        let depth = reader.read_i32()?;
        let clipping_depth = reader.read_i32()?;
        let color_matrix = ColorMatrix::deserialize(reader)?;
        let min_x = reader.read_f32()?;
        let max_x = reader.read_f32()?;
        let min_y = reader.read_f32()?;
        let max_y = reader.read_f32()?;
        let transformation_matrix = Matrix33::deserialize(reader)?;

        Ok(Self {
            char_id,
            char_index,
            depth,
            clipping_depth,
            transformation_matrix,
            color_matrix,
            min_x,
            max_x,
            min_y,
            max_y,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(self.char_id);
        builder.write_i32(self.char_index);
        builder.write_i32(self.depth);
        builder.write_i32(self.clipping_depth);
        self.color_matrix.serialize(builder)?;
        builder.write_f32(self.min_x);
        builder.write_f32(self.max_x);
        builder.write_f32(self.min_y);
        builder.write_f32(self.max_y);
        self.transformation_matrix.serialize(builder)?;
        Ok(())
    }
}
