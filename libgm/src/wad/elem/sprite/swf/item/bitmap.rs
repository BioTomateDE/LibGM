// SPDX-License-Identifier: GPL-3.0-only
use crate::gm_enum::gm_enum;
use crate::prelude::*;
use crate::wad::Blob;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::GMElement;
use crate::wad::parse::reader::DataReader;

#[derive(Debug, Clone, PartialEq)]
pub struct Data {
    pub bitmap_type: Type,
    pub width: i32,
    pub height: i32,
    pub ver_data: VersionData,
}

gm_enum!(Type {
    JpegNoHeader = 0,
    Jpeg = 1,
    JpegWithAlpha = 2,
    Png = 3,
    Gif = 4,
    Lossless8Bit = 5,
    Lossless15Bit = 6,
    Lossless24Bit = 7,
    Lossless8BitAlpha = 8,
    Lossless32Bit = 9,
});

impl GMElement for Data {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let bitmap_type: Type = reader.read_enum()?;
        let width = reader.read_i32()?;
        let height = reader.read_i32()?;

        let ver_data = if reader.general_info.version >= (2022, 1) {
            let tpe_index = reader.read_i32()?;
            VersionData::Post2022_1(VersionDataPost2022_1 { tpe_index })
        } else {
            let image_data_length = reader.read_count("YYSWF Bitmap Image Data")?;
            let alpha_data_length = reader.read_count("YYSWF Bitmap Alpha Data")?;
            let color_palette_data_length = reader.read_count("YYSWF Bitmap Color Palette Data")?;

            let image_data: Vec<u8> = reader
                .read_bytes_dyn(image_data_length)
                .ctx("reading Image Data of Bitmap Data")?
                .to_vec();
            let alpha_data: Vec<u8> = reader
                .read_bytes_dyn(alpha_data_length)
                .ctx("reading Alpha Data of Bitmap Data")?
                .to_vec();
            let color_palette_data: Vec<u8> = reader
                .read_bytes_dyn(color_palette_data_length)
                .ctx("reading Color Palette Data of Bitmap Data")?
                .to_vec();

            reader.align(4)?;
            VersionData::Pre2022_1(VersionDataPre2022_1 {
                image_data: Blob(image_data),
                alpha_data: Blob(alpha_data),
                color_palette_data: Blob(color_palette_data),
            })
        };

        Ok(Self { bitmap_type, width, height, ver_data })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_enum(self.bitmap_type);
        builder.write_i32(self.width);
        builder.write_i32(self.height);
        if builder.version() >= (2022, 1) {
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

#[derive(Debug, Clone, PartialEq)]
pub struct VersionDataPre2022_1 {
    pub image_data: Blob<Vec<u8>>,
    pub alpha_data: Blob<Vec<u8>>,
    pub color_palette_data: Blob<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VersionDataPost2022_1 {
    pub tpe_index: i32,
}
