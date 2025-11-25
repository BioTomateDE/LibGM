use crate::gamemaker::deserialize::reader::DataReader;
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::serialize::builder::DataBuilder;
use crate::prelude::*;
use crate::util::assert::assert_int;
use crate::util::init::num_enum_from;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::ops::{Deref, DerefMut};

/// GMS 2.3+
#[derive(Debug, Clone, Default)]
pub struct GMAnimationCurves {
    pub animation_curves: Vec<GMAnimationCurve>,
    pub exists: bool,
}

impl Deref for GMAnimationCurves {
    type Target = Vec<GMAnimationCurve>;
    fn deref(&self) -> &Self::Target {
        &self.animation_curves
    }
}

impl DerefMut for GMAnimationCurves {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.animation_curves
    }
}

impl GMChunkElement for GMAnimationCurves {
    const NAME: &'static str = "ACRV";

    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMAnimationCurves {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.align(4)?;
        assert_int("ACRV Version", 1, reader.read_u32()?)?;
        let animation_curves: Vec<GMAnimationCurve> =
            reader.read_pointer_list()?;
        Ok(Self { animation_curves, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.align(4);
        builder.write_i32(1); // ACRV version 1
        builder.write_pointer_list(&self.animation_curves)?;
        Ok(())
    }
}

/// An animation curve entry in a data file.
/// These were introduced in `GameMaker` 2.3.0.
#[derive(Debug, Clone, PartialEq)]
pub struct GMAnimationCurve {
    pub name: String,
    /// TODO: migrate to an enum
    pub graph_type: u32,
    pub channels: Vec<GMAnimationCurveChannel>,
}

impl GMElement for GMAnimationCurve {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name = reader.read_gm_string()?;
        let graph_type = reader.read_u32()?;
        let channels: Vec<GMAnimationCurveChannel> =
            reader.read_simple_list()?;
        Ok(GMAnimationCurve { name, graph_type, channels })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.name);
        builder.write_u32(self.graph_type.into());
        builder.write_simple_list(&self.channels)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMAnimationCurveChannel {
    pub name: String,
    pub curve_type: GMAnimationCurveType,
    pub iterations: u32,
    pub points: Vec<GMAnimationCurveChannelPoint>,
}

impl GMElement for GMAnimationCurveChannel {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name = reader.read_gm_string()?;
        let curve_type: GMAnimationCurveType =
            num_enum_from(reader.read_u32()?)?;
        let iterations = reader.read_u32()?;
        let points: Vec<GMAnimationCurveChannelPoint> =
            reader.read_simple_list()?;
        Ok(GMAnimationCurveChannel { name, curve_type, iterations, points })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.name);
        builder.write_u32(self.curve_type.into());
        builder.write_u32(self.iterations);
        builder.write_simple_list(&self.points)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMAnimationCurveChannelPoint {
    pub x: f32,
    pub y: f32, // Aka Value
    pub bezier_data: Option<PointBezierData>,
}

impl GMElement for GMAnimationCurveChannelPoint {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let x = reader.read_f32()?;
        let y = reader.read_f32()?;
        let mut bezier_data: Option<PointBezierData> = None;
        if reader.general_info.is_version_at_least((2, 3, 1)) {
            bezier_data = Some(PointBezierData::deserialize(reader)?);
        } else {
            reader.cur_pos += 4;
        };
        Ok(GMAnimationCurveChannelPoint { x, y, bezier_data })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_f32(self.x);
        builder.write_f32(self.y);

        if builder.is_gm_version_at_least((2, 3, 1)) {
            let bezier_data: &PointBezierData =
                self.bezier_data.as_ref().ok_or(
                    "Animation Curve Point's Bezier data not set in 2.3.1+",
                )?;
            bezier_data.serialize(builder)?;
        } else {
            builder.write_i32(0);
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

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_f32(self.x0);
        builder.write_f32(self.y0);
        builder.write_f32(self.x1);
        builder.write_f32(self.y1);
        Ok(())
    }
}

/// The curve type determines how points flow to each other in a channel.
#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum GMAnimationCurveType {
    /// Creates a linear progression between points.
    Linear = 0,
    /// Creates a smooth progression between points using catmull-rom spline interpolation.
    Smooth = 1,
    /// Creates a smooth curve defined by bezier control points.
    Bezier = 2,
}
