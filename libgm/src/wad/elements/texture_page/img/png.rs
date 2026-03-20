use std::io::Cursor;

use image::{DynamicImage, ImageFormat};

use crate::prelude::*;

#[cfg(feature = "png-image")]
pub fn decode(raw_png_data: &[u8]) -> Result<DynamicImage> {
    image::load_from_memory_with_format(raw_png_data, ImageFormat::Png)
        .context_src("decoding PNG Image")
}

#[cfg(not(feature = "png-image"))]
pub fn decode(_: &[u8]) -> Result<DynamicImage> {
    bail!("Crate feature `png-image` is disabled; cannot decode PNG image");
}

#[cfg(feature = "png-image")]
pub fn encode(dyn_img: &DynamicImage) -> Result<Vec<u8>> {
    let mut png_data: Vec<u8> = Vec::new();
    let mut cursor = Cursor::new(&mut png_data);
    dyn_img
        .write_to(&mut cursor, ImageFormat::Png)
        .context_src("encoding PNG Image")?;
    Ok(png_data)
}

#[cfg(not(feature = "png-image"))]
pub fn encode(_: &DynamicImage) -> Result<Vec<u8>> {
    bail!("Crate feature `png-image` is disabled; cannot encode PNG image");
}
