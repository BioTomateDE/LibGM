use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
    util::init::vec_with_capacity,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Data {
    pub fill_style1: i32,
    pub fill_style2: i32,
    pub line_style: i32,
    pub points: Vec<(f32, f32)>,
    pub lines: Vec<(i32, i32)>,
    pub triangles: Vec<i32>,
    pub line_points: Vec<(f32, f32)>,
    pub line_triangles: Vec<i32>,
    pub aa_lines: Vec<(i32, i32)>,
    pub aa_vectors: Vec<(f32, f32)>,
    pub line_aa_lines: Vec<(i32, i32)>,
    pub line_aa_vectors: Vec<(f32, f32)>,
}

impl GMElement for Data {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let fill_style1 = reader.read_i32()?;
        let fill_style2 = reader.read_i32()?;
        let line_style = reader.read_i32()?;

        let point_count = reader.read_u32()?;
        let line_count = reader.read_u32()?;
        let triangle_count = reader.read_u32()? * 3;
        let line_point_count = reader.read_u32()?;
        let line_triangle_count = reader.read_u32()? * 3;
        let aa_line_count = reader.read_u32()?;
        let aa_vector_count = reader.read_u32()?;
        let line_aa_line_count = reader.read_u32()?;
        let line_aa_vector_count = reader.read_u32()?;

        let mut points: Vec<(f32, f32)> = vec_with_capacity(point_count)?;
        let mut lines: Vec<(i32, i32)> = vec_with_capacity(line_count)?;
        let mut triangles: Vec<i32> = vec_with_capacity(triangle_count)?;
        let mut line_points: Vec<(f32, f32)> = vec_with_capacity(line_point_count)?;
        let mut line_triangles: Vec<i32> = vec_with_capacity(line_triangle_count)?;
        let mut aa_lines: Vec<(i32, i32)> = vec_with_capacity(aa_line_count)?;
        let mut aa_vectors: Vec<(f32, f32)> = vec_with_capacity(aa_vector_count)?;
        let mut line_aa_lines: Vec<(i32, i32)> = vec_with_capacity(line_aa_line_count)?;
        let mut line_aa_vectors: Vec<(f32, f32)> = vec_with_capacity(line_aa_vector_count)?;

        for _ in 0..point_count {
            points.push((reader.read_f32()?, reader.read_f32()?));
        }
        for _ in 0..line_count {
            lines.push((reader.read_i32()?, reader.read_i32()?));
        }
        for _ in 0..triangle_count {
            triangles.push(reader.read_i32()?);
        }
        for _ in 0..line_point_count {
            line_points.push((reader.read_f32()?, reader.read_f32()?));
        }
        for _ in 0..line_triangle_count {
            line_triangles.push(reader.read_i32()?);
        }
        for _ in 0..aa_line_count {
            aa_lines.push((reader.read_i32()?, reader.read_i32()?));
        }
        for _ in 0..aa_vector_count {
            aa_vectors.push((reader.read_f32()?, reader.read_f32()?));
        }
        for _ in 0..line_aa_line_count {
            line_aa_lines.push((reader.read_i32()?, reader.read_i32()?));
        }
        for _ in 0..line_aa_vector_count {
            line_aa_vectors.push((reader.read_f32()?, reader.read_f32()?));
        }

        Ok(Self {
            fill_style1,
            fill_style2,
            line_style,
            points,
            lines,
            triangles,
            line_points,
            line_triangles,
            aa_lines,
            aa_vectors,
            line_aa_lines,
            line_aa_vectors,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(self.fill_style1);
        builder.write_i32(self.fill_style2);
        builder.write_i32(self.line_style);

        builder.write_usize(self.points.len())?;
        builder.write_usize(self.lines.len())?;
        builder.write_usize(self.triangles.len() / 3)?;
        builder.write_usize(self.line_points.len())?;
        builder.write_usize(self.line_triangles.len() / 3)?;
        builder.write_usize(self.aa_lines.len())?;
        builder.write_usize(self.aa_vectors.len())?;
        builder.write_usize(self.line_aa_lines.len())?;
        builder.write_usize(self.line_aa_vectors.len())?;

        for (x, y) in &self.points {
            builder.write_f32(*x);
            builder.write_f32(*y);
        }
        for (x, y) in &self.lines {
            builder.write_i32(*x);
            builder.write_i32(*y);
        }
        for i in &self.triangles {
            builder.write_i32(*i);
        }
        for (x, y) in &self.line_points {
            builder.write_f32(*x);
            builder.write_f32(*y);
        }
        for i in &self.line_triangles {
            builder.write_i32(*i);
        }
        for (x, y) in &self.aa_lines {
            builder.write_i32(*x);
            builder.write_i32(*y);
        }
        for (x, y) in &self.aa_vectors {
            builder.write_f32(*x);
            builder.write_f32(*y);
        }
        for (x, y) in &self.line_aa_lines {
            builder.write_i32(*x);
            builder.write_i32(*y);
        }
        for (x, y) in &self.line_aa_vectors {
            builder.write_f32(*x);
            builder.write_f32(*y);
        }
        Ok(())
    }
}
