use crate::prelude::*;
use crate::wad::deserialize::reader::DataReader;
use crate::wad::elements::GMElement;
use crate::wad::elements::options::Constant;
use crate::wad::elements::options::Flags;
use crate::wad::elements::options::GMOptions;
use crate::wad::elements::texture_page_item::GMTexturePageItem;
use crate::wad::reference::GMRef;
use crate::wad::serialize::builder::DataBuilder;

pub fn parse(reader: &mut DataReader) -> Result<GMOptions> {
    let unknown1 = reader.read_u32()?;
    reader.assert_int(unknown1, 0x8000_0000, "Options Unknown Value 1")?;
    let unknown2 = reader.read_u32()?;
    reader.assert_int(unknown2, 2, "Options Unknown Value 2")?;
    let flags = Flags::deserialize(reader)?;
    let window_scale = reader.read_i32()?;
    let window_color = reader.read_u32()?;
    let color_depth = reader.read_u32()?;
    let resolution = reader.read_u32()?;
    let frequency = reader.read_u32()?;
    let vertex_sync = reader.read_i32()?;
    let priority = reader.read_i32()?;
    let back_image: Option<GMRef<GMTexturePageItem>> = reader.read_gm_texture_opt()?;
    let front_image: Option<GMRef<GMTexturePageItem>> = reader.read_gm_texture_opt()?;
    let load_image: Option<GMRef<GMTexturePageItem>> = reader.read_gm_texture_opt()?;
    let load_alpha = reader.read_u32()?;
    let constants: Vec<Constant> = reader.read_simple_list()?;

    Ok(GMOptions {
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
        exists: true,
    })
}

pub fn build(builder: &mut DataBuilder, options: &GMOptions) -> Result<()> {
    builder.write_u32(0x8000_0000); // unknown1
    builder.write_u32(2); //unknown2
    options.flags.serialize(builder)?;
    builder.write_i32(options.window_scale);
    builder.write_u32(options.window_color);
    builder.write_u32(options.color_depth);
    builder.write_u32(options.resolution);
    builder.write_u32(options.frequency);
    builder.write_i32(options.vertex_sync);
    builder.write_i32(options.priority);
    builder.write_pointer_opt(&options.back_image);
    builder.write_pointer_opt(&options.front_image);
    builder.write_pointer_opt(&options.load_image);
    builder.write_u32(options.load_alpha);
    builder.write_simple_list(&options.constants)?;
    Ok(())
}
