use crate::deserialize::all::GMData;
use crate::deserialize::fonts::{GMFont, GMFontGlyph, GMFontGlyphKerning};
use crate::deserialize::general_info::GMGeneralInfo;
use crate::serialize::all::DataBuilder;
use crate::serialize::chunk_writing::{ChunkBuilder, GMPointer};
use crate::serialize::sprites::align_writer;

pub fn build_chunk_font(data_builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    let mut builder = ChunkBuilder::new(data_builder, "FONT");

    let font_count: usize = gm_data.fonts.fonts_by_index.len();
    builder.write_usize(font_count);

    for i in 0..font_count {
        data_builder.write_pointer_placeholder(&mut builder, GMPointer::Font(i))?;
    }

    for (i, font) in gm_data.fonts.fonts_by_index.iter().enumerate() {
        data_builder.resolve_pointer(&mut builder, GMPointer::Font(i))?;
        build_font(data_builder, &mut builder, &gm_data.general_info, i, font)
            .map_err(|e| format!("{e} while building Font #{} with name \"{}\"", i, font.name.display(&gm_data.strings)))?;
    }

    builder.finish(data_builder)?;
    Ok(())
}


fn build_font(data_builder: &mut DataBuilder, builder: &mut ChunkBuilder, general_info: &GMGeneralInfo, font_index: usize, font: &GMFont) -> Result<(), String> {
    builder.write_gm_string(data_builder, &font.name)?;
    builder.write_gm_string(data_builder, &font.display_name)?;
    if general_info.is_version_at_least(2, 3, 0, 0) {   // {!!} i made this up; this doesn't exist in UTMT
        builder.write_f32(-font.em_size);
    } else {
        builder.write_u32(font.em_size as u32);
    }
    builder.write_bool32(font.bold);
    builder.write_bool32(font.italic);
    builder.write_u16(font.range_start);
    builder.write_u8(font.charset);
    builder.write_u8(font.anti_alias);
    builder.write_u32(font.range_end);
    data_builder.write_pointer_placeholder(builder, GMPointer::Texture(font.texture.index))?;
    builder.write_f32(font.scale_x);
    builder.write_f32(font.scale_y);

    if general_info.bytecode_version >= 17 {
        builder.write_i32(font.ascender_offset.ok_or("Ascender offset not set")?)
    }
    if general_info.is_version_at_least(2022, 2, 0, 0) {
        builder.write_u32(font.ascender.ok_or("Ascender not set")?)
    }
    if general_info.is_version_at_least(2023, 2, 0, 0) {
        builder.write_u32(font.sdf_spread.ok_or("SDF spread not set")?)
    }
    if general_info.is_version_at_least(2023, 6, 0, 0) {
        builder.write_u32(font.line_height.ok_or("Line height not set")?)
    }

    build_glyphs(data_builder, builder, general_info, &font.glyphs, font_index)?;

    if general_info.is_version_at_least(2024, 14, 0, 0) {
        align_writer(builder, 4, 0x00);
    }

    Ok(())
}


fn build_glyphs(data_builder: &mut DataBuilder, builder: &mut ChunkBuilder, general_info: &GMGeneralInfo, glyphs: &[GMFontGlyph], font_index: usize) -> Result<(), String> {
    builder.write_usize(glyphs.len());

    for i in 0..glyphs.len() {
        data_builder.write_pointer_placeholder(builder, GMPointer::FontGlyph(font_index, i))?;
    }

    for (i, glyph) in glyphs.iter().enumerate() {
        data_builder.resolve_pointer(builder, GMPointer::FontGlyph(font_index, i))?;

        let character: u16 = convert_char(glyph.character)
            .map_err(|e| format!("{e} for Glyph #{i}"))?;
        
        builder.write_u16(character);
        builder.write_u16(glyph.x);
        builder.write_u16(glyph.y);
        builder.write_u16(glyph.width);
        builder.write_u16(glyph.height);
        builder.write_i16(glyph.shift_modifier);
        builder.write_i16(glyph.offset);
        if general_info.is_version_at_least(2024, 11, 0, 0) {
            builder.write_i16(0);   // UnknownAlwaysZero
        }
        build_kernings(builder, &glyph.kernings)?;
    }

    Ok(())
}


fn build_kernings(builder: &mut ChunkBuilder, kernings: &Vec<GMFontGlyphKerning>) -> Result<(), String> {
    builder.write_u16(kernings.len() as u16);
    
    for kerning in kernings {
        builder.write_u16(convert_char(Some(kerning.character))?);
        builder.write_i16(kerning.shift_modifier);
    }
    
    Ok(())
}


fn convert_char(character: Option<char>) -> Result<u16, String> {
    match character {
        None => Ok(0),
        Some(ch) => {
            let number: u32 = ch.into();
            let number: u16 = u16::try_from(number)
                .map_err(|_| format!("Could not fit character '{ch}' (0x{number:08X}) into 16 bits (which is required by GameMaker)"))?;
            Ok(number)
        }
    }
}

