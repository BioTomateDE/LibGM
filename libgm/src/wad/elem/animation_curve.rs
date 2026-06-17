// SPDX-License-Identifier: GPL-3.0-only
pub mod channel;

pub use channel::Channel;

use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::gm_named_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::parse::reader::DataReader;

/// GMS 2.3+
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GMAnimationCurves {
    pub elems: Vec<Option<GMAnimationCurve>>,
    pub exists: bool,
}

gm_named_list_chunk!(ACRV, GMAnimationCurves, GMAnimationCurve, nullable);

impl GMElement for GMAnimationCurves {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.align(4)?;
        reader.read_gms2_chunk_version("ACRV Version")?;

        let elems: Vec<Option<GMAnimationCurve>> = reader.read_pointer_list_opt()?;
        Ok(Self { elems, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.align(4);
        builder.write_i32(1); // ACRV version 1
        builder.write_pointer_list_opt(&self.elems)?;
        Ok(())
    }
}

/// An animation curve entry in a data file.
/// These were introduced in GameMaker 2.3.0.
#[derive(Debug, Clone, PartialEq)]
pub struct GMAnimationCurve {
    pub name: GMRef<String>,
    pub channels: Vec<Channel>,
}

impl GMElement for GMAnimationCurve {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name = reader.read_gm_string()?;
        let graph_type = reader.read_u32()?;
        reader.assert_int(graph_type, 1, "Graph Type")?; // UTMT suggests this, lmk if this is wrong
        let channels: Vec<Channel> = reader.read_simple_list()?;
        Ok(Self { name, channels })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(self.name)?;
        builder.write_u32(1); // Graph Type
        builder.write_simple_list(&self.channels)?;
        Ok(())
    }
}
