// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::options::Constant;
use crate::wad::elem::options::OptionFlags;
use crate::wad::elem::options::Options;
use crate::wad::elem::texture_page_item::TexturePageItem;
use crate::wad::parse::reader::DataReader;
use crate::wad::reference::GMRef;

pub fn parse(reader: &mut DataReader) -> Result<Options> {
    let unknown1 = reader.read_u32()?;
    reader.assert_int(unknown1, 0x8000_0000, "Options Unknown Value 1")?;
    let unknown2 = reader.read_u32()?;
    reader.assert_int(unknown2, 2, "Options Unknown Value 2")?;
    let flags = reader.read_u64()?;
    let flags = OptionFlags::from_bits(flags)
        .ok_or_else(|| format!("Unknown OPTN Flags 0x{flags:016X}"))?;
    let window_scale = reader.read_i32()?;
    let window_color = reader.read_u32()?;
    let color_depth = reader.read_u32()?;
    let resolution = reader.read_u32()?;
    let frequency = reader.read_u32()?;
    let vertex_sync = reader.read_i32()?;
    let priority = reader.read_i32()?;
    let back_image: GMRef<TexturePageItem> = reader.read_gm_texture()?;
    let front_image: GMRef<TexturePageItem> = reader.read_gm_texture()?;
    let load_image: GMRef<TexturePageItem> = reader.read_gm_texture()?;
    let load_alpha = reader.read_u32()?;
    let constants: Vec<Constant> = reader.read_simple_list()?;

    Ok(Options {
        is_new_format: true,
        flags,
        window_scale,
        window_color,
        color_depth,
        resolution,
        frequency,
        vertex_sync,
        priority,
        back_image,
        front_image,
        load_image,
        load_alpha,
        constants,
    })
}

pub fn build(builder: &mut DataBuilder, options: &Options) -> Result<()> {
    builder.write_u32(0x8000_0000); // unknown1
    builder.write_u32(2); //unknown2
    builder.write_u64(options.flags.bits());
    builder.write_i32(options.window_scale);
    builder.write_u32(options.window_color);
    builder.write_u32(options.color_depth);
    builder.write_u32(options.resolution);
    builder.write_u32(options.frequency);
    builder.write_i32(options.vertex_sync);
    builder.write_i32(options.priority);
    builder.write_gm_texture(options.back_image)?;
    builder.write_gm_texture(options.front_image)?;
    builder.write_gm_texture(options.load_image)?;
    builder.write_u32(options.load_alpha);
    builder.write_simple_list(&options.constants)?;
    Ok(())
}
