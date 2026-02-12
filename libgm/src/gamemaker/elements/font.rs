mod glyph;
mod kerning;

pub use glyph::Glyph;
pub use kerning::Kerning;
use macros::named_list_chunk;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMElement, texture_page_item::GMTexturePageItem},
        reference::GMRef,
        serialize::builder::DataBuilder,
        version::LTSBranch,
    },
    prelude::*,
};

#[named_list_chunk("FONT")]
pub struct GMFonts {
    pub fonts: Vec<GMFont>,
    pub exists: bool,
}

impl GMElement for GMFonts {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let fonts: Vec<GMFont> = reader.read_pointer_list()?;

        if !reader.general_info.is_version_at_least((2024, 14)) {
            let verify: bool = reader.options.verify_constants;
            let padding: &[u8; 512] = reader.read_bytes_const().context("Reading FONT padding")?;
            if verify {
                verify_padding(padding)?;
            }
        }

        Ok(Self { fonts, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_pointer_list(&self.fonts)?;
        if !builder.is_version_at_least((2024, 14)) {
            let padding: [u8; 512] = generate_padding();
            builder.write_bytes(&padding);
        }

        Ok(())
    }
}

fn verify_padding(padding: &[u8; 512]) -> Result<()> {
    padding.iter().enumerate().try_for_each(|(i, &byte)| {
        let expected = match i {
            0..256 if i & 1 == 0 => (i >> 1) as u8,
            256..512 if i & 1 == 0 => 63,
            _ => 0,
        };

        if byte == expected {
            Ok(())
        } else {
            bail!("Invalid FONT padding at byte #{i}: expected 0x{expected:02X}, got 0x{byte:02X}")
        }
    })
}

#[must_use]
const fn generate_padding() -> [u8; 512] {
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

    padding
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMFont {
    /// The name of the font.
    pub name: String,

    /// The display name of the font.
    pub display_name: Option<String>,

    /// The font size in `Em`s.
    /// In GameMaker Studio 2.3+, this is stored as f32 instead of u32.
    pub em_size: FontSize,

    /// Whether to display the font in bold.
    pub bold: bool,

    /// Whether to display the font in italics.
    pub italic: bool,

    /// The start of the character range for this font.
    pub range_start: u16,

    /// TODO(doc): Currently unknown value.
    /// Possibly related to ranges? (aka normal, ascii, digits, letters)
    pub charset: u8,

    /// The level of antialiasing that is applied.
    /// GMS1 has 0-3 for different antialiasing levels.
    /// GMS2 has 0 and 1 for disabled/enabled.
    pub anti_alias: u8,

    /// The end of the character range for this font.
    pub range_end: u32,

    /// The [`GMTexturePageItem`] element that contains the texture for this font.
    pub texture: GMRef<GMTexturePageItem>,

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
    /// 0 means disabled.
    /// TODO(doc): what is spread, what is sdf?
    pub sdf_spread: Option<u32>,

    /// Was introduced in GM 2023.6.
    /// TODO(doc): give an explanation of what this does
    pub line_height: Option<u32>,

    /// The glyphs that this font uses.
    pub glyphs: Vec<Glyph>,
}

impl GMElement for GMFont {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: String = reader.read_gm_string()?;
        let display_name: Option<String> = reader.read_gm_string_opt()?;
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
        let texture: GMRef<GMTexturePageItem> = reader.read_gm_texture()?;
        let scale_x: f32 = reader.read_f32()?;
        let scale_y: f32 = reader.read_f32()?;
        let ascender_offset: Option<i32> = reader.deserialize_if_wad_version(17)?;
        let ascender: Option<u32> = reader.deserialize_if_gm_version((2022, 2))?;
        let sdf_spread: Option<u32> =
            reader.deserialize_if_gm_version((2023, 2, LTSBranch::PostLTS))?;
        let line_height: Option<u32> = reader.deserialize_if_gm_version((2023, 6))?;
        let glyphs: Vec<Glyph> = reader.read_pointer_list()?;
        if reader.general_info.is_version_at_least((2024, 14)) {
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
        builder.write_gm_string(&self.name);
        builder.write_gm_string_opt(&self.display_name);
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
        builder.write_if_wad_ver(&self.ascender_offset, "Ascender Offset", 17)?;
        builder.write_if_ver(&self.ascender, "Ascender", (2022, 2))?;
        builder.write_if_ver(
            &self.sdf_spread,
            "SDF Spread",
            (2023, 2, LTSBranch::PostLTS),
        )?;
        builder.write_if_ver(&self.line_height, "Line Height", (2023, 6))?;
        builder.write_pointer_list(&self.glyphs)?;
        if builder.is_version_at_least((2024, 14)) {
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
