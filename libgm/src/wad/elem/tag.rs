// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::ChunkName;
use crate::wad::elem::GMElement;
use crate::wad::parse::reader::DataReader;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Tags {
    pub elems: Vec<GMRef<String>>,
    pub asset_tags: Vec<AssetTags>,
}

impl GMChunk for Tags {
    const NAME: ChunkName = ChunkName::TAGS;
}

impl GMElement for Tags {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.align(4)?;
        reader.read_gms2_chunk_version("TAGS Version")?;
        let elems: Vec<GMRef<String>> = reader.read_simple_list()?;
        let asset_tags: Vec<AssetTags> = reader.read_pointer_list()?;

        Ok(Self { elems, asset_tags })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.align(4);
        builder.write_i32(1); // TAGS version
        builder.write_simple_list(&self.elems)?;
        builder.write_pointer_list(&self.asset_tags)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssetTags {
    pub id: i32,
    pub tags: Vec<GMRef<String>>,
}

impl GMElement for AssetTags {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let id = reader.read_i32()?;
        let tags: Vec<GMRef<String>> = reader.read_simple_list()?;
        Ok(Self { id, tags })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(self.id);
        builder.write_simple_list(&self.tags)?;
        Ok(())
    }
}
