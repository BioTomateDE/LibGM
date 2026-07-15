// SPDX-License-Identifier: GPL-3.0-only
mod glyph;
mod kerning;

pub use glyph::Glyph;
pub use kerning::Kerning;

use crate::prelude::*;
use crate::wad::Blob;
use crate::wad::GMVersion;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::gm_named_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::elem::texture_page_item::TexturePageItem;
use crate::wad::parse::reader::DataReader;
use crate::wad::reference::GMRef;

#[derive(Debug, Clone, PartialEq)]
pub struct Fonts {
    pub elems: Vec<Option<Font>>,
    pub padding: Blob<[u8; 512]>,
    pub exists: bool,
}

gm_named_list_chunk!(FONT, Fonts, Font, nullable);

impl GMElement for Fonts {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let elems: Vec<Option<Font>> = reader.read_pointer_list_opt()?;
        let padding = if reader.version < GMVersion::GM2024_14 {
            let bytes = reader
                .read_bytes_const()
                .ctx("Reading FONT padding bytes")?;
            Blob(*bytes)
        } else {
            generate_padding()
        };
        Ok(Self { elems, padding, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_pointer_list_opt(&self.elems)?;
        if builder.version() < GMVersion::GM2024_14 {
            builder.write_bytes(&*self.padding);
        }
        Ok(())
    }
}

impl Default for Fonts {
    fn default() -> Self {
        Self {
            elems: Vec::new(),
            padding: generate_padding(),
            exists: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Font {
    /// The name of the font.
    pub name: GMRef<String>,

    /// The display name of the font.
    pub display_name: GMRef<String>,

    /// The font size in `Em`s.
    /// In GameMaker Studio 2.3+, this is stored as f32 instead of u32.
    pub em_size: FontSize,

    /// Whether to display the font in bold.
    pub bold: bool,

    /// Whether to display the font in italics.
    pub italic: bool,

    /// The start of the character range for this font.
    pub range_start: u16,

    /// DOCME: Currently unknown value.
    /// Possibly related to ranges? (aka normal, ascii, digits, letters)
    pub charset: u8,

    /// The level of antialiasing that is applied.
    /// GMS1 has 0-3 for different antialiasing levels.
    /// GMS2 has 0 and 1 for disabled/enabled.
    pub anti_alias: u8,

    /// The end of the character range for this font.
    pub range_end: u32,

    /// The [`TexturePageItem`] element that contains the texture for this
    /// font.
    pub texture: GMRef<TexturePageItem>,

    /// The X Scale this font uses.
    pub scale_x: f32,

    /// The Y Scale this font uses.
    pub scale_y: f32,

    /// The maximum offset from the baseline to the top of the font.
    /// Exists since WAD 17, but seems to be only get checked in GM 2022.2+.
    pub ascender_offset: Option<i32>,

    /// Probably this: <https://en.wikipedia.org/wiki/Ascender_(typography)>; but needs investigation.
    /// Was introduced in GM 2022.2.
    pub ascender: Option<u32>,

    /// A spread value that's used for SDF rendering; was introduced in GM 2023.2.
    /// A value of 0 means it's disabled.
    /// DOCME: what is spread, what is sdf?
    pub sdf_spread: Option<u32>,

    /// Was introduced in GM 2023.6 (and 2022.0.3 LTS).
    /// DOCME: give an explanation of what this does
    pub line_height: Option<u32>,

    /// The glyphs that this font uses.
    pub glyphs: Vec<Glyph>,
}

impl GMElement for Font {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let display_name: GMRef<String> = reader.read_gm_string()?;
        let em_size = reader.read_u32()?; // Before GMS 2.3: int. after: float
        let em_size: FontSize = if em_size & (1 << 31) != 0 {
            // Since the float is always written negated, it has the first bit set.
            FontSize::Float(-f32::from_bits(em_size))
        } else {
            FontSize::Int(em_size)
        };
        let bold = reader.read_bool32()?;
        let italic = reader.read_bool32()?;
        let range_start = reader.read_u16()?;
        let charset = reader.read_u8()?;
        let anti_alias = reader.read_u8()?;
        let range_end = reader.read_u32()?;
        let texture: GMRef<TexturePageItem> = reader.read_gm_texture()?;
        let scale_x: f32 = reader.read_f32()?;
        let scale_y: f32 = reader.read_f32()?;
        let ascender_offset: Option<i32> = reader.deserialize_if_version(GMVersion::GMS2_2_1)?;
        let ascender: Option<u32> = reader.deserialize_if_version(GMVersion::GM2022_2)?;
        let sdf_spread: Option<u32> = if reader.version >= GMVersion::GM2023_2 {
            Some(reader.read_u32()?)
        } else {
            None
        };
        let line_height: Option<u32> =
            if reader.version >= GMVersion::GM2023_6 || reader.version == GMVersion::Lts2022_0_3 {
                Some(reader.read_u32()?)
            } else {
                None
            };
        let glyphs: Vec<Glyph> = reader.read_pointer_list()?;
        if reader.version >= GMVersion::GM2024_14 {
            reader.align(4)?;
        }

        Ok(Self {
            name,
            display_name,
            em_size,
            bold,
            italic,
            range_start,
            charset,
            anti_alias,
            range_end,
            texture,
            scale_x,
            scale_y,
            ascender_offset,
            ascender,
            sdf_spread,
            line_height,
            glyphs,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(self.name)?;
        builder.write_gm_string(self.display_name)?;
        match self.em_size {
            FontSize::Float(value) => builder.write_f32(-value),
            FontSize::Int(value) => builder.write_u32(value),
        }
        builder.write_bool32(self.bold);
        builder.write_bool32(self.italic);
        builder.write_u16(self.range_start);
        builder.write_u8(self.charset);
        builder.write_u8(self.anti_alias);
        builder.write_u32(self.range_end);
        builder.write_gm_texture(self.texture)?;
        builder.write_f32(self.scale_x);
        builder.write_f32(self.scale_y);
        builder.write_if_ver(
            &self.ascender_offset,
            "Ascender Offset",
            GMVersion::GMS2_2_1,
        )?;
        builder.write_if_ver(&self.ascender, "Ascender", GMVersion::GM2022_2)?;
        if builder.version() >= GMVersion::GM2023_2 {
            builder.write_u32(self.sdf_spread.ok_or("SDF Spread not set in 2023.2+")?);
        }
        if builder.version() >= GMVersion::GM2023_6 || builder.version() == GMVersion::Lts2022_0_3 {
            builder.write_u32(
                self.line_height
                    .ok_or("Line Height not set in 2023.6 (or LTS 2022.0.3)")?,
            );
        }
        builder.write_pointer_list(&self.glyphs)?;
        if builder.version() >= GMVersion::GM2024_14 {
            builder.align(4);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FontSize {
    Float(f32),
    Int(u32),
}

/// Generates standard padding bytes for the end of the `FONT` chunk.
#[must_use]
pub const fn generate_padding() -> Blob<[u8; 512]> {
    let mut padding = [0u8; 512];
    let mut i = 0;

    while i < 256 {
        padding[i] = if i & 1 == 0 { (i >> 1) as u8 } else { 0 };
        i += 1;
    }

    while i < 512 {
        padding[i] = if i & 1 == 0 { 63 } else { 0 };
        i += 1;
    }

    Blob(padding)
}
