use num_enum::{IntoPrimitive, TryFromPrimitive};
use crate::gamemaker::deserialize::{DataReader, GMRef};
use crate::gamemaker::element::{GMChunkElement, GMElement};
use crate::gamemaker::serialize::DataBuilder;

#[derive(Debug, Clone)]
pub struct GMAnimationCurves {
    pub animation_curves: Vec<GMAnimationCurve>,
    pub exists: bool,
}

impl GMChunkElement for GMAnimationCurves {
    fn empty() -> Self {
        Self { animation_curves: vec![], exists: false }
    }
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMAnimationCurves {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        reader.align(4)?;
        let version: i32 = reader.read_i32()?;
        if version != 1 {
            return Err(format!("Expected ACRV version 1 but got {version}"))
        }
        let animation_curves: Vec<GMAnimationCurve> = reader.read_pointer_list()?;
        Ok(Self { animation_curves, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }
        builder.align(4);
        builder.write_i32(1);  // ACRV version 1
        builder.write_pointer_list(&self.animation_curves)?;
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMAnimationCurve {
    pub name: GMRef<String>,
    pub graph_type: u32,
    pub channels: Vec<GMAnimationCurveChannel>,
}
impl GMElement for GMAnimationCurve {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let graph_type: u32 = reader.read_u32()?;
        let channels: Vec<GMAnimationCurveChannel> = reader.read_simple_list()?;
        Ok(GMAnimationCurve { name, graph_type, channels })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_gm_string(&self.name)?;
        builder.write_u32(self.graph_type.into());
        builder.write_simple_list(&self.channels)?;
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMAnimationCurveChannel {
    pub name: GMRef<String>,
    pub curve_type: GMAnimationCurveType,
    pub iterations: u32,
    pub points: Vec<GMAnimationCurveChannelPoint>,
}
impl GMElement for GMAnimationCurveChannel {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let curve_type: u32 = reader.read_u32()?;
        let curve_type: GMAnimationCurveType = curve_type.try_into()
            .map_err(|_| format!(
                "Invalid Curve Type {} for Animation Curve \"{}\" at absolute position {} in chunk '{}'",
                curve_type, reader.display_gm_str(name), reader.cur_pos, reader.chunk.name,
            ))?;
        let iterations: u32 = reader.read_u32()?;
        let points: Vec<GMAnimationCurveChannelPoint> = reader.read_simple_list()?;
        Ok(GMAnimationCurveChannel { name, curve_type, iterations, points })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_gm_string(&self.name)?;
        builder.write_u32(self.curve_type.into());
        builder.write_u32(self.iterations);
        builder.write_simple_list(&self.points)?;
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMAnimationCurveChannelPoint {
    pub x: f32,
    pub y: f32,     // aka Value
    pub bezier_data: Option<PointBezierData>,
}
impl GMElement for GMAnimationCurveChannelPoint {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let x: f32 = reader.read_f32()?;
        let y: f32 = reader.read_f32()?;
        let mut bezier_data: Option<PointBezierData> = None;
        if reader.general_info.is_version_at_least((2, 3, 1)) {
            bezier_data = Some(PointBezierData::deserialize(reader)?);
        } else {
            reader.cur_pos += 4;
        };
        Ok(GMAnimationCurveChannelPoint { x, y, bezier_data })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_f32(self.x);
        builder.write_f32(self.y);
        
        if builder.is_gm_version_at_least((2, 3, 1)) {
            let bezier_data: &PointBezierData = self.bezier_data.as_ref()
                .ok_or("Animation Curve Point: Bezier data not set in 2.3.1+")?;
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
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let x0: f32 = reader.read_f32()?;
        let y0: f32 = reader.read_f32()?;
        let x1: f32 = reader.read_f32()?;
        let y1: f32 = reader.read_f32()?;
        Ok(Self { x0, y0, x1, y1 })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_f32(self.x0);
        builder.write_f32(self.y0);
        builder.write_f32(self.x1);
        builder.write_f32(self.y1);
        Ok(())
    }
}


#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum GMAnimationCurveType {
    Linear = 0,
    Smooth = 1,
    // bezier missing idk
}

