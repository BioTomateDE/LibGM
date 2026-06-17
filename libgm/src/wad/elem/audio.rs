// SPDX-License-Identifier: GPL-3.0-only
use std::fmt;

use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::gm_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::parse::reader::DataReader;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GMAudios {
    pub elems: Vec<GMAudio>,
    pub exists: bool,
}

gm_list_chunk!(AUDO, GMAudios, GMAudio, direct);

impl GMElement for GMAudios {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let elems: Vec<GMAudio> = reader.read_pointer_list()?;
        Ok(Self { elems, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_pointer_list(&self.elems)?;
        Ok(())
    }
}

/// An embedded audio entry in a data file.
#[derive(Clone, PartialEq)]
pub struct GMAudio {
    /// The raw audio data of the embedded audio entry.
    /// This can be either WAV or OGG.
    pub data: Vec<u8>,
}

impl fmt::Debug for GMAudio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GMAudio").finish_non_exhaustive()
    }
}

impl GMElement for GMAudio {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let size = reader.read_u32()?;
        let data: Vec<u8> = reader.read_bytes_dyn(size)?.to_vec();
        Ok(Self { data })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_usize(self.data.len())?;
        builder.write_bytes(&self.data);
        Ok(())
    }

    fn deserialize_post_padding(reader: &mut DataReader, is_last: bool) -> Result<()> {
        if !is_last {
            reader.align(4)?;
        }
        Ok(())
    }

    fn serialize_post_padding(&self, builder: &mut DataBuilder, is_last: bool) -> Result<()> {
        if !is_last {
            builder.align(4);
        }
        Ok(())
    }
}
