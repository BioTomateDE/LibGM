use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::deserialize::fonts::{GMFont, GMFontGlyph};
use crate::export_mod::export::{edit_field, edit_field_option, GModData, ModUnorderedRef};
use crate::export_mod::unordered_list::{export_changes_unordered_list, AModUnorderedListChanges, GModUnorderedListChanges};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModFont {
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
    pub glyphs: AModUnorderedListChanges<ModFontGlyph>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModFontGlyph {
    pub character: Option<char>,
    pub x: Option<u16>,
    pub y: Option<u16>,
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub shift_modifier: Option<i16>,
    pub offset: Option<i16>,
}


impl GModData<'_, '_> {
    pub fn convert_font_additions(&self, gm_fonts: &Vec<GMFont>) -> Result<Vec<ModFont>, String> {
        let mut mod_fonts: Vec<ModFont> = Vec::with_capacity(gm_fonts.len());

        for font in gm_fonts {
            mod_fonts.push(ModFont {
                name: Some(self.resolve_string_ref(&font.name)?),
                display_name: Some(self.resolve_string_ref(&font.display_name)?),
                em_size: Some(font.em_size),
                bold: Some(font.bold),
                italic: Some(font.italic),
                range_start: Some(font.range_start),
                charset: Some(font.charset),
                anti_alias: Some(font.anti_alias),
                range_end: Some(font.range_end),
                texture: Some(self.resolve_texture_ref(&font.texture)?),
                scale_x: Some(font.scale_x),
                scale_y: Some(font.scale_y),
                ascender_offset: font.ascender_offset,
                ascender: font.ascender,
                sdf_spread: font.sdf_spread,
                line_height: font.line_height,
                glyphs: AModUnorderedListChanges {additions: self.convert_font_glyphs_additions(&font.glyphs)?, edits: HashMap::new()},
            });
        }

        Ok(mod_fonts)
    }

    pub fn convert_fonts(&self, changes: &GModUnorderedListChanges<GMFont>) -> Result<AModUnorderedListChanges<ModFont>, String> {
        let additions: Vec<ModFont> = self.convert_font_additions(&changes.additions)?;
        let mut edits: HashMap<usize, ModFont> = HashMap::new();

        for (index, (original, modified)) in &changes.edits {
            edits.insert(*index, ModFont {
                name: edit_field(&self.resolve_string_ref(&original.name)?, &self.resolve_string_ref(&modified.name)?),
                display_name: edit_field(&self.resolve_string_ref(&original.display_name)?, &self.resolve_string_ref(&modified.display_name)?),
                em_size: edit_field(&original.em_size, &modified.em_size),
                bold: edit_field(&original.bold, &modified.bold),
                italic: edit_field(&original.italic, &modified.italic),
                range_start: edit_field(&original.range_start, &modified.range_start),
                charset: edit_field(&original.charset, &modified.charset),
                anti_alias: edit_field(&original.anti_alias, &modified.anti_alias),
                range_end: edit_field(&original.range_end, &modified.range_end),
                texture: edit_field(&self.resolve_texture_ref(&original.texture)?, &self.resolve_texture_ref(&modified.texture)?),
                scale_x: edit_field(&original.scale_x, &modified.scale_x),
                scale_y: edit_field(&original.scale_y, &modified.scale_y),
                ascender_offset: *edit_field_option(&original.ascender_offset, &modified.ascender_offset),
                ascender: *edit_field_option(&original.ascender, &modified.ascender),
                sdf_spread: *edit_field_option(&original.sdf_spread, &modified.sdf_spread),
                line_height: *edit_field_option(&original.line_height, &modified.line_height),
                glyphs: self.convert_font_glyphs(&export_changes_unordered_list(&original.glyphs, &modified.glyphs)?)?,
            });
        }

        Ok(AModUnorderedListChanges { additions, edits })
    }

    fn convert_font_glyphs_additions(&self, gm_glyphs: &Vec<GMFontGlyph>) -> Result<Vec<ModFontGlyph>, String> {
        let mut mod_glyphs: Vec<ModFontGlyph> = Vec::with_capacity(gm_glyphs.len());

        for glyph in gm_glyphs {
            mod_glyphs.push(ModFontGlyph {
                // character: Some(glyph.character),
                character: None,    //TODO
                x: Some(glyph.x),
                y: Some(glyph.y),
                width: Some(glyph.width),
                height: Some(glyph.height),
                shift_modifier: Some(glyph.shift_modifier),
                offset: Some(glyph.offset),
            })
        }

        Ok(mod_glyphs)
    }

    fn convert_font_glyphs(&self, changes: &GModUnorderedListChanges<GMFontGlyph>) -> Result<AModUnorderedListChanges<ModFontGlyph>, String> {
        let additions: Vec<ModFontGlyph> = self.convert_font_glyphs_additions(&changes.additions)?;
        let mut edits: HashMap<usize, ModFontGlyph> = HashMap::new();

        for (index, (original, modified)) in &changes.edits {
            edits.insert(*index, ModFontGlyph {
                // character: edit_field(&original.character, &modified.character),
                character: None, //TODO
                x: edit_field(&original.x, &modified.x),
                y: edit_field(&original.y, &modified.y),
                width: edit_field(&original.width, &modified.width),
                height: edit_field(&original.height, &modified.height),
                shift_modifier: edit_field(&original.shift_modifier, &modified.shift_modifier),
                offset: edit_field(&original.offset, &modified.offset),
            });
        }

        Ok(AModUnorderedListChanges { additions, edits })
    }
}