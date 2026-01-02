use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
};

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
        let data = if reader.general_info.is_version_at_least((2023, 1)) {
            let texture_entry_length = reader.read_u32()?;
            Data::Post2023_1(texture_entry_length)
        } else {
            let size = reader.read_u32()?;
            let texture_blob: Vec<u8> = reader.read_bytes_dyn(size)?.to_vec();
            Data::Pre2023_1(texture_blob)
        };
        Ok(Self { page_width, page_height, data })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_u32(self.page_width);
        builder.write_u32(self.page_height);
        if builder.is_version_at_least((2023, 1)) {
            if let Data::Post2023_1(texture_entry_length) = self.data {
                builder.write_u32(texture_entry_length);
            } else {
                bail!(
                    "Expected Post2023_1 Spine Texture Entry data but got Pre2023_1 for some reason"
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
    /// > implementing `serde::Serialize` for this probably isn't the best idea
    Pre2023_1(Vec<u8>),
    /// Texture entry count.
    Post2023_1(u32),
}
