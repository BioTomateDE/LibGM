use crate::deserialize::all::GMData;
use crate::deserialize::fonts::{GMFont, GMFontGlyph};
use crate::serialize::all::{build_chunk, DataBuilder};
use crate::serialize::chunk_writing::{ChunkBuilder, GMPointer};

pub fn build_chunk_font(data_builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    let mut builder: ChunkBuilder = ChunkBuilder { raw_data: Vec::new(), chunk_name: "FONT", abs_pos: data_builder.len() };

    let font_count: usize = gm_data.fonts.fonts_by_index.len();
    builder.write_usize(font_count);

    for i in 0..font_count {
        data_builder.push_pointer_placeholder(&mut builder, GMPointer::font(i))?;
    }

    for i in 0..font_count {
        let font: &GMFont = &gm_data.fonts.fonts_by_index[i];
        data_builder.push_pointer_resolve(&mut builder, GMPointer::font(i))?;
        builder.write_literal_string(&font.name.resolve(&gm_data.strings.strings_by_index)?)?;
        builder.write_literal_string(&font.display_name.resolve(&gm_data.strings.strings_by_index)?)?;
        builder.write_u32(font.em_size);
        builder.write_u32(if font.bold {1} else {0});
        builder.write_u32(if font.italic {1} else {0});
        builder.write_u16(font.range_start);
        builder.write_u8(font.charset);
        builder.write_u8(font.anti_alias);
        builder.write_u32(font.range_end);
        data_builder.push_pointer_placeholder(&mut builder, GMPointer::texture(font.texture.index))?;
        builder.write_f32(font.scale_x);
        builder.write_f32(font.scale_y);

        // maybe check version instead of checking if not none? (probably ok if handled correctly by mod versions)
        if let Some(number) = font.ascender_offset {
            builder.write_i32(number);
        };
        if let Some(number) = font.ascender {
            builder.write_u32(number);
        };
        if let Some(number) = font.sdf_spread {
            builder.write_u32(number);
        };
        if let Some(number) = font.line_height {
            builder.write_u32(number);
        };

        build_glyphs(data_builder, &mut builder, &font.glyphs, i, font.name.resolve(&gm_data.strings.strings_by_index)?)?;
    }

    build_chunk(data_builder, builder)?;
    Ok(())
}


fn build_glyphs(data_builder: &mut DataBuilder, builder: &mut ChunkBuilder, glyphs: &[GMFontGlyph], font_index: usize, font_name: &str) -> Result<(), String> {
    builder.write_usize(glyphs.len());

    for i in 0..glyphs.len() {
        data_builder.push_pointer_placeholder(builder, GMPointer::font_glyph(font_index, i))?;
    }

    for (i, glyph) in glyphs.iter().enumerate() {
        data_builder.push_pointer_resolve(builder, GMPointer::font_glyph(font_index, i))?;

        let character: u16 = glyph.character.try_into()
            .map_err(|e| format!(
                "Unable to fit character '{}' (0x{:04X}) into 16 bits \
                (which is required by GameMaker) for glyph with index {} \
                in font \"{}\". Error message: {e}",
                glyph.character, glyph.character as u32, i, font_name,
            ))?;
        
        builder.write_u16(character);
        builder.write_u16(glyph.x);
        builder.write_u16(glyph.y);
        builder.write_u16(glyph.width);
        builder.write_u16(glyph.height);
        builder.write_i16(glyph.shift_modifier);
        builder.write_i16(glyph.offset);
        builder.write_u16(0);      // kerning
    }

    Ok(())
}

