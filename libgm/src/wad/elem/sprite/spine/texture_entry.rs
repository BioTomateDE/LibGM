// SPDX-License-Identifier: GPL-3.0-only

use crate::prelude::*;
use crate::wad::Blob;
use crate::wad::GMVersion;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::GMElement;
use crate::wad::parse::reader::DataReader;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextureEntry {
    pub page_width: u32,
    pub page_height: u32,
    pub data: Data,
}

impl GMElement for TextureEntry {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let page_width = reader.read_u32()?;
        let page_height = reader.read_u32()?;
        let data = if reader.version >= GMVersion::GM2023_1 {
            let texture_entry_length = reader.read_u32()?;
            Data::Post2023_1(texture_entry_length)
        } else {
            let size = reader.read_u32()?;
            let texture_blob: Vec<u8> = reader.read_bytes_dyn(size)?.to_vec();
            Data::Pre2023_1(Blob(texture_blob))
        };
        Ok(Self { page_width, page_height, data })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_u32(self.page_width);
        builder.write_u32(self.page_height);
        if builder.version() >= GMVersion::GM2023_1 {
            if let Data::Post2023_1(texture_entry_length) = self.data {
                builder.write_u32(texture_entry_length);
            } else {
                bail!(
                    "Expected Post2023_1 Spine Texture Entry data but got Pre2023_1 for some \
                     reason"
                );
            }
        } else if let Data::Pre2023_1(ref texture_blob) = self.data {
            builder.write_usize(texture_blob.len())?;
            builder.write_bytes(texture_blob);
        } else {
            bail!("Expected Pre2023_1 Spine Texture Entry data but got Post2023_1 for some reason");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Data {
    /// Texture blob raw data.
    Pre2023_1(Blob<Vec<u8>>),

    /// Texture entry count.
    Post2023_1(u32),
}
