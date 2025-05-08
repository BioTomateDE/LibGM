use crate::export_mod::export::ModUnorderedRef;

#[derive(Debug, Clone)]
pub struct ModFont {
    pub name: Option<ModUnorderedRef>, // String
    pub display_name: Option<ModUnorderedRef>,  // String
    pub em_size: Option<u32>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub range_start: Option<u16>,
    pub charset: Option<u8>,
    pub anti_alias: Option<u8>,
    pub range_end: Option<u32>,
    pub texture: Option<u32>,   // Replace with TexturePageItem when available
    pub scale_x: Option<f32>,
    pub scale_y: Option<f32>,
    pub ascender_offset: Option<i32>,
    pub ascender: Option<u32>,
    pub sdf_spread: Option<u32>,
    pub line_height: Option<u32>,
    pub glyphs: Option<Vec<ModFontGlyph>>,
}

#[derive(Debug, Clone)]
pub struct ModFontGlyph {
    pub character: Option<char>,
    pub x: Option<u16>,
    pub y: Option<u16>,
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub shift_modifier: Option<i16>,
    pub offset: Option<i16>,
}

