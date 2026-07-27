// SPDX-License-Identifier: GPL-3.0-only

use crate::gm_enum::gm_enum;
use crate::prelude::*;
use crate::wad::GMVersion;
use crate::wad::elem::GMElement;
use crate::wad::load::reader::DataReader;
use crate::wad::save::writer::DataWriter;

gm_enum!(
/// The curve type determines how points flow to each other in a channel.
CurveType {
    /// Creates a linear progression between points.
    Linear = 0,
    /// Creates a smooth progression between points using catmull-rom spline
    /// interpolation.
    Smooth = 1,
    /// Creates a smooth curve defined by bezier control points.
    Bezier = 2,
});

#[derive(Debug, Clone, PartialEq)]
pub struct Channel {
    pub name: GMRef<String>,
    pub curve_type: CurveType,
    pub iterations: u32,
    pub points: Vec<Point>,
}

impl GMElement for Channel {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name = reader.read_gm_string()?;
        let curve_type: CurveType = reader.read_enum()?;
        let iterations = reader.read_u32()?;
        let points: Vec<Point> = reader.read_simple_list()?;
        Ok(Self { name, curve_type, iterations, points })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_gm_string(self.name)?;
        writer.write_enum(self.curve_type);
        writer.write_u32(self.iterations);
        writer.write_simple_list(&self.points)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    pub x: f32,
    /// also known as "value"
    pub y: f32,
    pub bezier_data: Option<PointBezierData>,
}

impl GMElement for Point {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let x = reader.read_f32()?;
        let y = reader.read_f32()?;
        let mut bezier_data: Option<PointBezierData> = None;
        if reader.version >= GMVersion::GMS2_3_1 {
            bezier_data = Some(PointBezierData::deserialize(reader)?);
        } else {
            reader.cur_pos += 4;
        }
        Ok(Self { x, y, bezier_data })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_f32(self.x);
        writer.write_f32(self.y);

        if writer.version() >= GMVersion::GMS2_3_1 {
            let bezier_data: &PointBezierData = self
                .bezier_data
                .as_ref()
                .ok_or("Animation Curve Point's Bezier data not set in 2.3.1+")?;
            writer.write_elem(bezier_data)?;
        } else {
            writer.write_i32(0);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PointBezierData {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

impl GMElement for PointBezierData {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let x0 = reader.read_f32()?;
        let y0 = reader.read_f32()?;
        let x1 = reader.read_f32()?;
        let y1 = reader.read_f32()?;
        Ok(Self { x0, y0, x1, y1 })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_f32(self.x0);
        writer.write_f32(self.y0);
        writer.write_f32(self.x1);
        writer.write_f32(self.y1);
        Ok(())
    }
}
