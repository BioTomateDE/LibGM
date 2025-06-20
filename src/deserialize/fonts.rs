use crate::deserialize::chunk_reading::{GMChunkElement, GMElement, DataReader, GMRef};
use crate::deserialize::general_info::GMVersionLTS;
use crate::deserialize::texture_page_items::GMTexturePageItem;

#[derive(Debug, Clone)]
pub struct GMFonts {
    pub fonts: Vec<GMFont>,
    pub exists: bool,
}
impl GMChunkElement for GMFonts {
    fn empty() -> Self {
        Self { fonts: vec![], exists: false }
    }
}
impl GMElement for GMFonts {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let fonts: Vec<GMFont> = reader.read_pointer_list()?;
        Ok(Self { fonts, exists: true })
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMFont {
    pub name: GMRef<String>,
    pub display_name: Option<GMRef<String>>,
    pub em_size: GMFontSize,
    pub bold: bool,
    pub italic: bool,
    pub range_start: u16,
    pub charset: u8,
    pub anti_alias: u8,
    pub range_end: u32,
    pub texture: GMRef<GMTexturePageItem>,
    pub scale_x: f32,
    pub scale_y: f32,
    pub ascender_offset: Option<i32>,
    pub ascender: Option<u32>,
    pub sdf_spread: Option<u32>,
    pub line_height: Option<u32>,
    pub glyphs: Vec<GMFontGlyph>,
}
impl GMElement for GMFont {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let display_name: Option<GMRef<String>> = reader.read_gm_string_opt()?;
        let em_size: u32 = reader.read_u32()?;   // before GMS 2.3: int. after: float
        let em_size: GMFontSize = if em_size & (1 << 31) != 0 {    // since the float is always written negated, it has the first bit set.
            GMFontSize::Float(-f32::from_bits(em_size))
        } else {
            GMFontSize::Int(em_size)
        };
        let bold: bool = reader.read_bool32()?;
        let italic: bool = reader.read_bool32()?;
        let range_start: u16 = reader.read_u16()?;
        let charset: u8 = reader.read_u8()?;
        let anti_alias: u8 = reader.read_u8()?;
        let range_end: u32 = reader.read_u32()?;
        let texture: GMRef<GMTexturePageItem> = reader.read_gm_texture()?;
        let scale_x: f32 = reader.read_f32()?;
        let scale_y: f32 = reader.read_f32()?;

        let mut ascender_offset: Option<i32> = None;
        let mut ascender: Option<u32> = None;
        let mut sdf_spread: Option<u32> = None;
        let mut line_height: Option<u32> = None;

        if reader.general_info.bytecode_version >= 17 {
            ascender_offset = Some(reader.read_i32()?);
        }
        if reader.general_info.is_version_at_least((2022, 2, 0, 0)) {
            ascender = Some(reader.read_u32()?);
        }
        if reader.general_info.is_version_at_least((2023, 2, 0, 0)) && reader.general_info.version.lts == GMVersionLTS::Post2022_0 {
            sdf_spread = Some(reader.read_u32()?);
        }
        if reader.general_info.is_version_at_least((2023, 6, 0, 0)) {
            line_height = Some(reader.read_u32()?);
        }
        let glyphs: Vec<GMFontGlyph> = reader.read_simple_list_short()?;
        if reader.general_info.is_version_at_least((2024, 14, 0, 0)) {
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
            scale_x,
            scale_y,
            ascender_offset,
            ascender,
            sdf_spread,
            line_height,
            glyphs,
        })
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMFontGlyph {
    pub character: Option<char>,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub shift_modifier: i16,
    pub offset: i16,
    pub kernings: Vec<GMFontGlyphKerning>,
}
impl GMElement for GMFontGlyph {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let character: u16 = reader.read_u16()?;
        let character: Option<char> = if character == 0 { None } else {
            Some(char::from_u32(character.into()).ok_or_else(|| format!("Invalid UTF-8 character with code point {0} (0x{0:04X})", character))?)
        };
        let x: u16 = reader.read_u16()?;
        let y: u16 = reader.read_u16()?;
        let width: u16 = reader.read_u16()?;
        let height: u16 = reader.read_u16()?;
        let shift_modifier: i16 = reader.read_i16()?;
        let offset: i16 = reader.read_i16()?;    // potential assumption according to utmt
        if reader.general_info.is_version_at_least((2024, 11, 0, 0)) {
            let _unknown_always_zero = reader.read_i16();
        }
        let kernings: Vec<GMFontGlyphKerning> = reader.read_simple_list()?;

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
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMFontGlyphKerning {
    pub character: char,
    pub shift_modifier: i16,
}
impl GMElement for GMFontGlyphKerning {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let character: u16 = reader.read_u16()?;
        if character == 0 {
            return Err("Character not set (code point is zero)".to_string())
        }
        let character: char = char::from_u32(character.into())
            .ok_or_else(|| format!("Invalid UTF-8 character with code point {0} (0x{0:04X})", character))?;
        let shift_modifier: i16 = reader.read_i16()?;
        Ok(GMFontGlyphKerning { character, shift_modifier })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GMFontSize {
    Float(f32),
    Int(u32),
}

