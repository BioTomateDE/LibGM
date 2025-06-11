use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::deserialize::fonts::{GMFont, GMFontGlyph};
use crate::export_mod::export::{edit_field, edit_field_option, GModData, ModUnorderedRef};
use crate::export_mod::unordered_list::{export_changes_unordered_list, EditUnorderedList, GModUnorderedListChanges};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddFont {
    pub name: ModUnorderedRef, // String
    pub display_name: ModUnorderedRef,  // String
    pub em_size: f32,
    pub bold: bool,
    pub italic: bool,
    pub range_start: u16,
    pub charset: u8,
    pub anti_alias: u8,
    pub range_end: u32,
    pub texture: ModUnorderedRef,   // Texture Page Item
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditFont {
    pub name: Option<ModUnorderedRef>, // String
    pub display_name: Option<ModUnorderedRef>,  // String
    pub em_size: Option<f32>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub range_start: Option<u16>,
    pub charset: Option<u8>,
    pub anti_alias: Option<u8>,
    pub range_end: Option<u32>,
    pub texture: Option<ModUnorderedRef>,   // Texture Page Item
    pub scale_x: Option<f32>,
    pub scale_y: Option<f32>,
    pub ascender_offset: Option<i32>,
    pub ascender: Option<u32>,
    pub sdf_spread: Option<u32>,
    pub line_height: Option<u32>,
    pub glyphs: EditUnorderedList<AddFontGlyph, EditFontGlyph>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditFontGlyph {
    pub character: Option<char>,
    pub x: Option<u16>,
    pub y: Option<u16>,
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub shift_modifier: Option<i16>,
    pub offset: Option<i16>,
}


impl GModData<'_, '_> {
    pub fn convert_font_additions(&self, gm_fonts: &[GMFont]) -> Result<Vec<AddFont>, String> {
        self.convert_additions(gm_fonts, |i| Ok(AddFont {
            name: self.resolve_string_ref(&i.name)?,
            display_name: self.resolve_string_ref(&i.display_name)?,
            em_size: i.em_size,
            bold: i.bold,
            italic: i.italic,
            range_start: i.range_start,
            charset: i.charset,
            anti_alias: i.anti_alias,
            range_end: i.range_end,
            texture: self.resolve_texture_ref(&i.texture)?,
            scale_x: i.scale_x,
            scale_y: i.scale_y,
            ascender_offset: i.ascender_offset,
            ascender: i.ascender,
            sdf_spread: i.sdf_spread,
            line_height: i.line_height,
            glyphs: self.convert_font_glyphs_additions(&i.glyphs)?,
        }))
    }

    pub fn convert_fonts(&self, changes: &GModUnorderedListChanges<GMFont>) -> Result<EditUnorderedList<AddFont, EditFont>, String> {
        self.convert_edits(&changes, |i| self.convert_font_additions(i), |o, m| {
            Ok(EditFont {
                name: edit_field(&self.resolve_string_ref(&o.name)?, &self.resolve_string_ref(&m.name)?),
                display_name: edit_field(&self.resolve_string_ref(&o.display_name)?, &self.resolve_string_ref(&m.display_name)?),
                em_size: edit_field(&o.em_size, &m.em_size),
                bold: edit_field(&o.bold, &m.bold),
                italic: edit_field(&o.italic, &m.italic),
                range_start: edit_field(&o.range_start, &m.range_start),
                charset: edit_field(&o.charset, &m.charset),
                anti_alias: edit_field(&o.anti_alias, &m.anti_alias),
                range_end: edit_field(&o.range_end, &m.range_end),
                texture: edit_field(&self.resolve_texture_ref(&o.texture)?, &self.resolve_texture_ref(&m.texture)?),
                scale_x: edit_field(&o.scale_x, &m.scale_x),
                scale_y: edit_field(&o.scale_y, &m.scale_y),
                ascender_offset: edit_field_option(&o.ascender_offset, &m.ascender_offset),
                ascender: edit_field_option(&o.ascender, &m.ascender),
                sdf_spread: edit_field_option(&o.sdf_spread, &m.sdf_spread),
                line_height: edit_field_option(&o.line_height, &m.line_height),
                glyphs: self.convert_font_glyphs(&export_changes_unordered_list(&o.glyphs, &m.glyphs)?)?,
            })
        })
    }

    fn convert_font_glyphs_additions(&self, gm_glyphs: &[GMFontGlyph]) -> Result<Vec<AddFontGlyph>, String> {
        let mut mod_glyphs: Vec<AddFontGlyph> = Vec::with_capacity(gm_glyphs.len());

        for i in gm_glyphs {
            mod_glyphs.push(AddFontGlyph {
                character: i.character,
                x: i.x,
                y: i.y,
                width: i.width,
                height: i.height,
                shift_modifier: i.shift_modifier,
                offset: i.offset,
            })
        }

        Ok(mod_glyphs)
    }

    fn convert_font_glyphs(&self, changes: &GModUnorderedListChanges<GMFontGlyph>) -> Result<EditUnorderedList<AddFontGlyph, EditFontGlyph>, String> {
        let additions: Vec<AddFontGlyph> = self.convert_font_glyphs_additions(changes.additions)?;
        let mut edits: HashMap<usize, EditFontGlyph> = HashMap::new();

        for (index, (original, modified)) in &changes.edits {
            edits.insert(*index, EditFontGlyph {
                character: edit_field_option(&original.character, &modified.character),
                x: edit_field(&original.x, &modified.x),
                y: edit_field(&original.y, &modified.y),
                width: edit_field(&original.width, &modified.width),
                height: edit_field(&original.height, &modified.height),
                shift_modifier: edit_field(&original.shift_modifier, &modified.shift_modifier),
                offset: edit_field(&original.offset, &modified.offset),
            });
        }

        Ok(EditUnorderedList { additions, edits })
    }
}

