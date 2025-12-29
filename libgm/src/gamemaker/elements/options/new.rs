use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{
            GMElement,
            options::{Constant, Flags, GMOptions},
            texture_page_item::GMTexturePageItem,
        },
        reference::GMRef,
        serialize::builder::DataBuilder,
    },
    prelude::*,
};

pub fn parse(reader: &mut DataReader) -> Result<GMOptions> {
    let unknown1 = reader.read_u32()?;
    let unknown2 = reader.read_u32()?;
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
        unknown1,
        unknown2,
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
    builder.write_u32(options.unknown1);
    builder.write_u32(options.unknown2);
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
