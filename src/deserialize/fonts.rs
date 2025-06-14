use std::collections::HashMap;
use crate::deserialize::chunk_reading::{GMChunk, GMRef};
use crate::deserialize::general_info::GMGeneralInfo;
use crate::deserialize::sprites::align_reader;
use crate::deserialize::strings::GMStrings;
use crate::deserialize::texture_page_items::{GMTexturePageItem, GMTextures};

#[derive(Debug, Clone, PartialEq)]
pub struct GMFont {
    pub name: GMRef<String>,
    pub display_name: Option<GMRef<String>>,
    pub em_size: f32,
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

#[derive(Debug, Clone, PartialEq)]
pub struct GMFontGlyphKerning {
    pub character: char,
    pub shift_modifier: i16,
}

#[derive(Debug, Clone)]
pub struct GMFonts {
    pub abs_pos_to_ref: HashMap<usize, GMRef<GMFont>>,  // convert absolute position/pointer in data.win to font ref
    pub fonts_by_index: Vec<GMFont>,                    // fonts by index/order in chunk FONT
}

pub fn parse_chunk_font(chunk: &mut GMChunk, general_info: &GMGeneralInfo, strings: &GMStrings, textures: &GMTextures) -> Result<GMFonts, String> {
    chunk.cur_pos = 0;
    let font_count: usize = chunk.read_usize_count()?;
    let mut font_starting_positions: Vec<usize> = Vec::with_capacity(font_count);
    for _ in 0..font_count {
        let start_position: usize = chunk.read_usize_pos()? - chunk.abs_pos;
        font_starting_positions.push(start_position);
    }

    let mut fonts_by_index: Vec<GMFont> = Vec::with_capacity(font_count);
    let mut abs_pos_to_ref: HashMap<usize, GMRef<GMFont>> = HashMap::new();
    for (i, start_position) in font_starting_positions.iter().enumerate() {
        chunk.cur_pos = *start_position;

        let name: GMRef<String> = chunk.read_gm_string(&strings)?;
        let display_name: Option<GMRef<String>> = chunk.read_gm_string_optional(&strings)?;
        let em_size: u32 = chunk.read_u32()?;   // before GMS 2.3: int. after: float
        let em_size: f32 = if em_size & (1 << 31) != 0 {    // since the float is always written negated, it has the first bit set.
            -f32::from_bits(em_size)
        } else {
            em_size as f32
        };
        let bold: bool = chunk.read_bool32()?;
        let italic: bool = chunk.read_bool32()?;
        let range_start: u16 = chunk.read_u16()?;
        let charset: u8 = chunk.read_u8()?;
        let anti_alias: u8 = chunk.read_u8()?;
        let range_end: u32 = chunk.read_u32()?;
        let texture_abs_pos: usize = chunk.read_usize_pos()?;
        let texture: GMRef<GMTexturePageItem> = textures.abs_pos_to_ref.get(&texture_abs_pos).ok_or_else(|| format!(
            "Could not find texture with absolute position {} for Font with name \"{}\" at position {} in chunk 'FONT'", 
            texture_abs_pos, name.display(strings), start_position,
        ))?.clone();
        let scale_x: f32 = chunk.read_f32()?;
        let scale_y: f32 = chunk.read_f32()?;

        let mut ascender_offset: Option<i32> = None;
        let mut ascender: Option<u32> = None;
        let mut sdf_spread: Option<u32> = None;
        let mut line_height: Option<u32> = None;

        if general_info.bytecode_version >= 17 {
            ascender_offset = Some(chunk.read_i32()?);
        }
        if general_info.is_version_at_least(2022, 2, 0, 0) {
            ascender = Some(chunk.read_u32()?);
        }
        if general_info.is_version_at_least(2023, 2, 0, 0) {    // TODO non LTS {~~}
            sdf_spread = Some(chunk.read_u32()?);
        }
        if general_info.is_version_at_least(2023, 6, 0, 0) {
            line_height = Some(chunk.read_u32()?);
        }

        let glyphs: Vec<GMFontGlyph> = parse_glyphs(chunk, general_info)
            .map_err(|e| format!("{e} of font #{i} with name \"{}\"", name.display(strings)))?;
        
        if general_info.is_version_at_least(2024, 14, 0, 0) {
            align_reader(chunk, 4, 0x00)?;
        }

        let font: GMFont = GMFont {
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
        };
        abs_pos_to_ref.insert(start_position + chunk.abs_pos, GMRef::new(i));
        fonts_by_index.push(font);
    }
    Ok(GMFonts {
        abs_pos_to_ref,
        fonts_by_index,
    })
}


fn parse_glyphs(chunk: &mut GMChunk, general_info: &GMGeneralInfo) -> Result<Vec<GMFontGlyph>, String> {
    let glyph_count: usize = chunk.read_usize_count()?;
    let mut glyph_starting_positions: Vec<usize> = Vec::with_capacity(glyph_count);

    for _ in 0..glyph_count {
        let start_position: usize = chunk.read_usize_pos()? - chunk.abs_pos;
        glyph_starting_positions.push(start_position);
    }

    let mut glyphs: Vec<GMFontGlyph> = Vec::with_capacity(glyph_count);
    for (i, start_position) in glyph_starting_positions.iter().enumerate() {
        chunk.cur_pos = *start_position;

        let character: i16 = chunk.read_i16()?;
        let character: Option<char> = convert_char(character).map_err(|_| format!(
            "Invalid UTF-8 character with code point {character} (0x{character:04X}) at absolute position {} in chunk 'FONT' for glyph #{i}",
            chunk.abs_pos + chunk.cur_pos,
        ))?;
        let x: u16 = chunk.read_u16()?;
        let y: u16 = chunk.read_u16()?;
        let width: u16 = chunk.read_u16()?;
        let height: u16 = chunk.read_u16()?;
        let shift_modifier: i16 = chunk.read_i16()?;
        let offset: i16 = chunk.read_i16()?;    // potential assumption idk
        if general_info.is_version_at_least(2024, 11, 0, 0) {
            let _unknown_always_zero = chunk.read_i16();
        }
        let kernings: Vec<GMFontGlyphKerning> = parse_kernings(chunk)
            .map_err(|e| format!("{e} for Glyph #{i}"))?;

        let glyph: GMFontGlyph = GMFontGlyph {
            character,
            x,
            y,
            width,
            height,
            shift_modifier,
            offset,
            kernings,
        };
        glyphs.push(glyph);
    }

    Ok(glyphs)
}


fn parse_kernings(chunk: &mut GMChunk) -> Result<Vec<GMFontGlyphKerning>, String> {
    let kerning_count: usize = chunk.read_u16()? as usize;
    let mut kernings: Vec<GMFontGlyphKerning> = Vec::with_capacity(kerning_count);
    
    for i in 0..kerning_count {
        let character: i16 = chunk.read_i16()?;
        let character: char = convert_char(character)
            .map_err(|_| format!("Invalid UTF-8 character with code point {character} (0x{character:04X}) for Kerning #{i}"))?
            .ok_or_else(|| format!("Character not set (code point is zero) for Kerning #{i}"))?;
        let shift_modifier: i16 = chunk.read_i16()?;
        
        kernings.push(GMFontGlyphKerning {
            character,
            shift_modifier,
        })
    }
    
    Ok(kernings)
}


fn convert_char(codepoint: i16) -> Result<Option<char>, ()> {
    if codepoint == 0 {
        return Ok(None);
    }
    char::from_u32(codepoint as u32).map(Some).ok_or(())
}


