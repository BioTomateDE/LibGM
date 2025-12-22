pub mod channel;

pub use channel::Channel;
use macros::named_list_chunk;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
};

/// GMS 2.3+
#[named_list_chunk("ACRV")]
pub struct GMAnimationCurves {
    pub animation_curves: Vec<GMAnimationCurve>,
    pub exists: bool,
}

impl GMElement for GMAnimationCurves {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.align(4)?;
        reader.read_gms2_chunk_version("ACRV Version")?;

        let animation_curves: Vec<GMAnimationCurve> = reader.read_pointer_list()?;
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
/// These were introduced in GameMaker 2.3.0.
#[derive(Debug, Clone, PartialEq)]
pub struct GMAnimationCurve {
    pub name: String,
    /// This field may change in the future.
    /// TODO: migrate to an enum
    pub graph_type: u32,
    pub channels: Vec<Channel>,
}

impl GMElement for GMAnimationCurve {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name = reader.read_gm_string()?;
        let graph_type = reader.read_u32()?;
        let channels: Vec<Channel> = reader.read_simple_list()?;
        Ok(Self { name, graph_type, channels })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.name);
        builder.write_u32(self.graph_type);
        builder.write_simple_list(&self.channels)?;
        Ok(())
    }
}
