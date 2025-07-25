use serde::{Deserialize, Serialize};
use crate::modding::export::{convert_additions, edit_field, edit_field_convert, edit_field_convert_option, edit_field_option, ModExporter, ModRef};
use crate::modding::ordered_list::{export_changes_ordered_list, DataChange};
use crate::gamemaker::elements::fonts::{GMFontGlyph, GMFontGlyphKerning, GMFontSize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddFont {
    pub name: ModRef, // String
    pub display_name: Option<ModRef>,  // String
    pub em_size: f32,
    pub bold: bool,
    pub italic: bool,
    pub range_start: u16,
    pub charset: u8,
    pub anti_alias: u8,
    pub range_end: u32,
    pub texture: ModRef,   // Texture Page Item
    pub scale_x: f32,
    pub scale_y: f32,
    pub ascender_offset: Option<i32>,
    pub ascender: Option<u32>,
    pub sdf_spread: Option<u32>,
    pub line_height: Option<u32>,
    pub glyphs: Vec<AddFontGlyph>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddFontGlyph {
    pub character: Option<char>,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub shift_modifier: i16,
    pub offset: i16,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditFont {
    pub name: Option<ModRef>, // String
    pub display_name: Option<ModRef>,  // String
    pub em_size: Option<f32>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub range_start: Option<u16>,
    pub charset: Option<u8>,
    pub anti_alias: Option<u8>,
    pub range_end: Option<u32>,
    pub texture: Option<ModRef>,   // Texture Page Item
    pub scale_x: Option<f32>,
    pub scale_y: Option<f32>,
    pub ascender_offset: Option<i32>,
    pub ascender: Option<u32>,
    pub sdf_spread: Option<u32>,
    pub line_height: Option<u32>,
    pub glyphs: Vec<DataChange<AddFontGlyph, EditFontGlyph>>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditFontGlyph {
    pub character: Option<char>,
    pub x: Option<u16>,
    pub y: Option<u16>,
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub shift_modifier: Option<i16>,
    pub offset: Option<i16>,
    pub kernings: Vec<DataChange<ModKerning, ModKerning>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModKerning {
    pub character: char,
    pub shift_modifier: i16,
}


impl ModExporter<'_, '_> {
    pub fn export_fonts(&self) -> Result<Vec<DataChange<AddFont, EditFont>>, String> {
        export_changes_ordered_list(
            &self.original_data.fonts.fonts,
            &self.modified_data.fonts.fonts,
            |i| Ok(AddFont {
                name: self.convert_string_ref(&i.name)?,
                display_name: self.convert_string_ref_opt(&i.display_name)?,
                em_size: convert_font_size(&i.em_size),
                bold: i.bold,
                italic: i.italic,
                range_start: i.range_start,
                charset: i.charset,
                anti_alias: i.anti_alias,
                range_end: i.range_end,
                texture: self.convert_texture_ref(&i.texture)?,
                scale_x: i.scale_x,
                scale_y: i.scale_y,
                ascender_offset: i.ascender_offset,
                ascender: i.ascender,
                sdf_spread: i.sdf_spread,
                line_height: i.line_height,
                glyphs: convert_additions(&i.glyphs, add_font_glyph)?,
            }),
            |o, m| Ok(EditFont {
                name: edit_field_convert(&o.name, &m.name, |r| self.convert_string_ref(r))?,
                display_name: edit_field_convert_option(&o.display_name, &m.display_name, |r| self.convert_string_ref(r))?.flatten(),
                em_size: edit_field(&convert_font_size(&o.em_size), &convert_font_size(&m.em_size)),
                bold: edit_field(&o.bold, &m.bold),
                italic: edit_field(&o.italic, &m.italic),
                range_start: edit_field(&o.range_start, &m.range_start),
                charset: edit_field(&o.charset, &m.charset),
                anti_alias: edit_field(&o.anti_alias, &m.anti_alias),
                range_end: edit_field(&o.range_end, &m.range_end),
                texture: edit_field_convert(&o.texture, &m.texture, |r| self.convert_texture_ref(r))?,
                scale_x: edit_field(&o.scale_x, &m.scale_x),
                scale_y: edit_field(&o.scale_y, &m.scale_y),
                ascender_offset: edit_field_option(&o.ascender_offset, &m.ascender_offset).flatten(),
                ascender: edit_field_option(&o.ascender, &m.ascender).flatten(),
                sdf_spread: edit_field_option(&o.sdf_spread, &m.sdf_spread).flatten(),
                line_height: edit_field_option(&o.line_height, &m.line_height).flatten(),
                glyphs: export_changes_ordered_list(&o.glyphs, &m.glyphs, add_font_glyph, edit_font_glyph)?,
            }),
        )
    }
}


fn add_font_glyph(i: &GMFontGlyph) -> Result<AddFontGlyph, String> {
    Ok(AddFontGlyph { 
        character: i.character, 
        x: i.x,
        y: i.y,
        width: i.width,
        height: i.height,
        shift_modifier: i.shift_modifier, 
        offset: i.offset,
    })
}

fn edit_font_glyph(o: &GMFontGlyph, m: &GMFontGlyph) -> Result<EditFontGlyph, String> {
    Ok(EditFontGlyph {
        character: edit_field_option(&o.character, &m.character).flatten(),
        x: edit_field(&o.x, &m.x),
        y: edit_field(&o.y, &m.y),
        width: edit_field(&o.width, &m.width),
        height: edit_field(&o.height, &m.height),
        shift_modifier: edit_field(&o.shift_modifier, &m.shift_modifier),
        offset: edit_field(&o.offset, &m.offset),
        kernings: export_changes_ordered_list(
            &o.kernings,
            &m.kernings,
            convert_font_kerning,
            |_, m| convert_font_kerning(m),
        )?,
    })
}

fn convert_font_size(i: &GMFontSize) -> f32 {
    match i {
        GMFontSize::Float(float) => *float,
        GMFontSize::Int(int) => *int as f32,
    }
}

fn convert_font_kerning(i: &GMFontGlyphKerning) -> Result<ModKerning, String> {
    Ok(ModKerning {
        character: i.character,
        shift_modifier: i.shift_modifier,
    })
}

