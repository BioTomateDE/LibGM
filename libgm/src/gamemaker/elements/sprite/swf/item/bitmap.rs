use std::fmt;

use macros::num_enum;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
    util::init::num_enum_from,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Data {
    pub bitmap_type: Type,
    pub width: i32,
    pub height: i32,
    pub ver_data: VersionData,
}

#[num_enum(i32)]
pub enum Type {
    TypeJPEGNoHeader,
    TypeJPEG,
    TypeJPEGWithAlpha,
    TypePNG,
    TypeGIF,
    TypeLossless8bit,
    TypeLossless15bit,
    TypeLossless24bit,
    TypeLossless8bitA,
    TypeLossless32bit,
}

impl GMElement for Data {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let bitmap_type: Type = num_enum_from(reader.read_i32()?)?;
        let width = reader.read_i32()?;
        let height = reader.read_i32()?;

        let ver_data = if reader.general_info.is_version_at_least((2022, 1)) {
            let tpe_index = reader.read_i32()?;
            VersionData::Post2022_1(VersionDataPost2022_1 { tpe_index })
        } else {
            let image_data_length = reader.read_count("YYSWF Bitmap Image Data")?;
            let alpha_data_length = reader.read_count("YYSWF Bitmap Alpha Data")?;
            let color_palette_data_length = reader.read_count("YYSWF Bitmap Color Palette Data")?;

            let image_data: Vec<u8> = reader
                .read_bytes_dyn(image_data_length)
                .context("reading Image Data of Bitmap Data")?
                .to_vec();
            let alpha_data: Vec<u8> = reader
                .read_bytes_dyn(alpha_data_length)
                .context("reading Alpha Data of Bitmap Data")?
                .to_vec();
            let color_palette_data: Vec<u8> = reader
                .read_bytes_dyn(color_palette_data_length)
                .context("reading Color Palette Data of Bitmap Data")?
                .to_vec();

            reader.align(4)?;
            VersionData::Pre2022_1(VersionDataPre2022_1 {
                image_data,
                alpha_data,
                color_palette_data,
            })
        };

        Ok(Self { bitmap_type, width, height, ver_data })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(self.bitmap_type.into());
        builder.write_i32(self.width);
        builder.write_i32(self.height);
        if builder.is_version_at_least((2022, 1)) {
            if let VersionData::Post2022_1(ref data) = self.ver_data {
                builder.write_i32(data.tpe_index);
            } else {
                bail!("Sprite YYSWF Bitmap Data: TPE Index not set in Post 2022.1+");
            }
        } else if let VersionData::Pre2022_1(ref data) = self.ver_data {
            builder.write_usize(data.image_data.len())?;
            builder.write_usize(data.alpha_data.len())?;
            builder.write_usize(data.color_palette_data.len())?;
            builder.write_bytes(&data.image_data);
            builder.write_bytes(&data.alpha_data);
            builder.write_bytes(&data.color_palette_data);
            builder.align(4);
        } else {
            bail!("Sprite YYSWF Bitmap Data: version specific data not set in Pre 2022.1+");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VersionData {
    Pre2022_1(VersionDataPre2022_1),
    Post2022_1(VersionDataPost2022_1),
}

#[derive(Clone, PartialEq)]
pub struct VersionDataPre2022_1 {
    pub image_data: Vec<u8>,
    pub alpha_data: Vec<u8>,
    pub color_palette_data: Vec<u8>,
}

impl fmt::Debug for VersionDataPre2022_1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("VersionDataPre2022_1")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VersionDataPost2022_1 {
    pub tpe_index: i32,
}
