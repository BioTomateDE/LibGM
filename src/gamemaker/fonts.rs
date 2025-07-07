use crate::gm_deserialize::{GMChunkElement, GMElement, DataReader, GMRef};
use crate::gamemaker::general_info::LTSBranch::Post2022_0;
use crate::gamemaker::texture_page_items::GMTexturePageItem;
use crate::gm_serialize::{DataBuilder, GMSerializeIfVersion};

#[derive(Debug, Clone)]
pub struct GMFonts {
    pub fonts: Vec<GMFont>,
    pub padding: Option<[u8; 512]>,
    pub exists: bool,
}
impl GMChunkElement for GMFonts {
    fn empty() -> Self {
        Self { fonts: vec![], padding: None, exists: false }
    }
    fn exists(&self) -> bool {
        self.exists
    }
}
impl GMElement for GMFonts {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let fonts: Vec<GMFont> = reader.read_pointer_list()?;
        
        let mut padding: Option<[u8; 512]> = None;
        if !reader.general_info.is_version_at_least((2024, 14)) {
            padding = Some(reader.read_bytes_const()?.clone())
        }
        
        Ok(Self { fonts, padding, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }
        
        builder.write_pointer_list(&self.fonts)?;

        if !builder.is_gm_version_at_least((2024, 14)) {
            let Some(padding) = self.padding else {
                return Err("FONT Chunk padding not set before 2024.14 (this could've been a warning probably since there is a fallback)".to_string())
            };
            builder.write_bytes(&padding);
        }

        Ok(())
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

        let ascender_offset: Option<i32> = reader.deserialize_if_bytecode_version(17)?;
        let ascender: Option<u32> = reader.deserialize_if_gm_version((2022, 2))?;
        let sdf_spread: Option<u32> = reader.deserialize_if_gm_version((2023, 2, Post2022_0))?;
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
            scale_x,
            scale_y,
            ascender_offset,
            ascender,
            sdf_spread,
            line_height,
            glyphs,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
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
        builder.write_f32(self.scale_x);
        builder.write_f32(self.scale_y);
        self.ascender_offset.serialize_if_bytecode_ver(builder, "Ascender Offset", 17)?;
        self.ascender.serialize_if_gm_ver(builder, "Ascender", (2022, 2))?;
        self.sdf_spread.serialize_if_gm_ver(builder, "SDF Spread", (2023, 2))?;
        self.line_height.serialize_if_gm_ver(builder, "Line Height", (2023, 6))?;
        builder.write_pointer_list(&self.glyphs)?;
        if builder.is_gm_version_at_least((2024, 14)) {
            builder.align(4);
        }
        Ok(())
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
        if reader.general_info.is_version_at_least((2024, 11)) {
            let unknown_always_zero: i16 = reader.read_i16()?;
            if unknown_always_zero != 0 {
                return Err(format!("Unknown Always Zero in Font Glyph with character {:?} has value {}", character, unknown_always_zero))
            }
        }
        let kernings: Vec<GMFontGlyphKerning> = reader.read_simple_list_short()?;

        Ok(GMFontGlyph { character, x, y, width, height, shift_modifier, offset, kernings })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
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
            builder.write_u16(0);   // UnknownAlwaysZero
        }
        builder.write_simple_list_short(&self.kernings)?;
        Ok(())
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

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
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

