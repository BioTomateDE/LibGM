use crate::gm_deserialize::{DataReader, GMChunkElement, GMElement};
use crate::gm_serialize::DataBuilder;

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


#[derive(Debug, Clone)]
pub struct GMAnimationCurve {

}
impl GMElement for GMAnimationCurve {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let name = reader.read_gm_string()?;
        // v TODO enum
        let graph_type: i32 = reader.read_i32()?;
        let channels: Vec<GMAnimationCurveChannel> = reader.read_simple_list()?;
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        todo!()
    }
}

