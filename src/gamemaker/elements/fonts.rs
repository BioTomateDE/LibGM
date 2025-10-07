use crate::gamemaker::deserialize::{DataReader, GMRef};
use crate::gamemaker::elements::texture_page_items::GMTexturePageItem;
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::gm_version::LTSBranch;
use crate::gamemaker::serialize::DataBuilder;
use crate::gamemaker::serialize::traits::GMSerializeIfVersion;
use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct GMFonts {
    pub fonts: Vec<GMFont>,
    pub padding: Option<[u8; 512]>,
    pub exists: bool,
}
impl GMChunkElement for GMFonts {
    fn stub() -> Self {
        Self { fonts: vec![], padding: None, exists: false }
    }
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMFonts {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let fonts: Vec<GMFont> = reader.read_pointer_list()?;

        let mut padding: Option<[u8; 512]> = None;
        if !reader.general_info.is_version_at_least((2024, 14)) {
            padding = Some(reader.read_bytes_const()?.clone())
        }

        Ok(Self { fonts, padding, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        if !self.exists {
            return Ok(());
        }

        builder.write_pointer_list(&self.fonts)?;

        if !builder.is_gm_version_at_least((2024, 14)) {
            let Some(padding) = self.padding else {
                bail!(
                    "FONT Chunk padding not set before 2024.14 (this could've been a warning probably since there is a fallback)"
                );
            };
            builder.write_bytes(&padding);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMFont {
    /// The name of the font.
    pub name: GMRef<String>,

    /// The display name of the font.
    pub display_name: Option<GMRef<String>>,

    /// The font size in `Em`s.
    /// In GameMaker Studio 2.3+, this is stored as f32 instead of u32.
    pub em_size: GMFontSize,

    /// Whether to display the font in bold.
    pub bold: bool,

    /// Whether to display the font in italics.
    pub italic: bool,

    /// The start of the character range for this font.
    pub range_start: u16,

    /// TODO: Currently unknown value. Possibly related to ranges? (aka normal, ascii, digits, letters)
    pub charset: u8,

    /// The level of antialiasing that is applied.
    /// GMS1 has 0-3 for different antialiasing levels.
    /// GMS2 has 0 and 1 for disabled/enabled.
    pub anti_alias: u8,

    /// The end of the character range for this font.
    pub range_end: u32,

    /// The `[GMTexturePageItem]` object that contains the texture for this font.
    pub texture: GMRef<GMTexturePageItem>,

    /// The X and Y Scale this font uses.
    pub scale: (f32, f32),

    /// The maximum offset from the baseline to the top of the font.
    /// Exists since bytecode 17, but seems to be only get checked in GM 2022.2+.
    pub ascender_offset: Option<i32>,

    /// Probably this: https://en.wikipedia.org/wiki/Ascender_(typography); but needs investigation.
    /// Was introduced in GM 2022.2.
    pub ascender: Option<u32>,

    /// A spread value that's used for SDF rendering; was introduced in GM 2023.2.
    /// 0 means disabled.
    /// TODO: what is spread, what is sdf?
    pub sdf_spread: Option<u32>,

    /// Was introduced in GM 2023.6.
    /// TODO: give an explanation of what this does
    pub line_height: Option<u32>,

    /// The glyphs that this font uses.
    pub glyphs: Vec<GMFontGlyph>,
}

impl GMElement for GMFont {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let display_name: Option<GMRef<String>> = reader.read_gm_string_opt()?;
        let em_size = reader.read_u32()?; // Before GMS 2.3: int. after: float
        let em_size: GMFontSize = if em_size & (1 << 31) != 0 {
            // Since the float is always written negated, it has the first bit set.
            GMFontSize::Float(-f32::from_bits(em_size))
        } else {
            GMFontSize::Int(em_size)
        };
        let bold = reader.read_bool32()?;
        let italic = reader.read_bool32()?;
        let range_start = reader.read_u16()?;
        let charset = reader.read_u8()?;
        let anti_alias = reader.read_u8()?;
        let range_end = reader.read_u32()?;
        let texture: GMRef<GMTexturePageItem> = reader.read_gm_texture()?;
        let scale: (f32, f32) = (reader.read_f32()?, reader.read_f32()?);
        let ascender_offset: Option<i32> = reader.deserialize_if_bytecode_version(17)?;
        let ascender: Option<u32> = reader.deserialize_if_gm_version((2022, 2))?;
        let sdf_spread: Option<u32> = reader.deserialize_if_gm_version((2023, 2, LTSBranch::PostLTS))?;
        let line_height: Option<u32> = reader.deserialize_if_gm_version((2023, 6))?;
        let glyphs: Vec<GMFontGlyph> = reader.read_pointer_list()?;
        if reader.general_info.is_version_at_least((2024, 14)) {
            reader.align(4)?;
        }

        Ok(GMFont {
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
            scale,
            ascender_offset,
            ascender,
            sdf_spread,
            line_height,
            glyphs,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.name)?;
        builder.write_gm_string_opt(&self.display_name)?;
        match self.em_size {
            GMFontSize::Float(value) => builder.write_f32(-value),
            GMFontSize::Int(value) => builder.write_u32(value),
        }
        builder.write_bool32(self.bold);
        builder.write_bool32(self.italic);
        builder.write_u16(self.range_start);
        builder.write_u8(self.charset);
        builder.write_u8(self.anti_alias);
        builder.write_u32(self.range_end);
        builder.write_gm_texture(&self.texture)?;
        builder.write_f32(self.scale.0);
        builder.write_f32(self.scale.1);
        self.ascender_offset
            .serialize_if_bytecode_ver(builder, "Ascender Offset", 17)?;
        self.ascender.serialize_if_gm_ver(builder, "Ascender", (2022, 2))?;
        self.sdf_spread
            .serialize_if_gm_ver(builder, "SDF Spread", (2023, 2, LTSBranch::PostLTS))?;
        self.line_height
            .serialize_if_gm_ver(builder, "Line Height", (2023, 6))?;
        builder.write_pointer_list(&self.glyphs)?;
        if builder.is_gm_version_at_least((2024, 14)) {
            builder.align(4);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMFontGlyph {
    /// The character this glyph represents.
    pub character: Option<char>,

    /// The x position in the [`GMFont`].`texture` where the glyph can be found.
    pub x: u16,

    /// The y position in the [`GMFont`].`texture` where the glyph can be found.
    pub y: u16,

    /// The width of the glyph in pixels.
    pub width: u16,

    /// The height of the glyph in pixels.
    pub height: u16,

    /// The number of pixels to shift right when advancing to the next character.
    pub shift_modifier: i16,

    /// The number of pixels to horizontally offset the rendering of this glyph.
    pub offset: i16,

    /// The kerning for each glyph.
    pub kernings: Vec<GMFontGlyphKerning>,
}
impl GMElement for GMFontGlyph {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let character = reader.read_u16()?;
        let character: Option<char> = if character == 0 {
            None
        } else {
            Some(
                char::from_u32(character.into())
                    .ok_or_else(|| format!("Invalid UTF-8 character with code point {0} (0x{0:04X})", character))?,
            )
        };
        let x = reader.read_u16()?;
        let y = reader.read_u16()?;
        let width = reader.read_u16()?;
        let height = reader.read_u16()?;
        let shift_modifier = reader.read_i16()?;
        let offset = reader.read_i16()?; // Potential assumption according to utmt
        if reader.general_info.is_version_at_least((2024, 11)) {
            let unknown_always_zero = reader.read_i16()?;
            if unknown_always_zero != 0 {
                bail!(
                    "Unknown Always Zero in Font Glyph with character {:?} has value {}",
                    character,
                    unknown_always_zero
                );
            }
        }
        let kernings: Vec<GMFontGlyphKerning> = reader.read_simple_list_short()?;

        Ok(GMFontGlyph {
            character,
            x,
            y,
            width,
            height,
            shift_modifier,
            offset,
            kernings,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        if let Some(char) = self.character {
            let codepoint: u32 = char.into();
            builder.write_u16(codepoint as u16);
        } else {
            builder.write_u16(0);
        }
        builder.write_u16(self.x);
        builder.write_u16(self.y);
        builder.write_u16(self.width);
        builder.write_u16(self.height);
        builder.write_i16(self.shift_modifier);
        builder.write_i16(self.offset);
        if builder.is_gm_version_at_least((2024, 11)) {
            builder.write_u16(0); // UnknownAlwaysZero
        }
        builder.write_simple_list_short(&self.kernings)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMFontGlyphKerning {
    /// The preceding character.
    pub character: char,
    /// An amount of pixels to add to the existing [`GMFontGlyph`].`shift_modifier`.
    pub shift_modifier: i16,
}
impl GMElement for GMFontGlyphKerning {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let character = reader.read_u16()?;
        if character == 0 {
            bail!("Character not set (code point is zero)");
        }
        let character: char = char::from_u32(character.into())
            .ok_or_else(|| format!("Invalid UTF-8 character with code point {0} (0x{0:04X})", character))?;
        let shift_modifier = reader.read_i16()?;
        Ok(GMFontGlyphKerning { character, shift_modifier })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_u16(u32::from(self.character) as u16);
        builder.write_i16(self.shift_modifier);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GMFontSize {
    Float(f32),
    Int(u32),
}
