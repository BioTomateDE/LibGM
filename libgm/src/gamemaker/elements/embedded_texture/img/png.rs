use image::DynamicImage;

use crate::prelude::*;

#[cfg(feature = "png-image")]
pub fn decode(raw_png_data: &[u8]) -> Result<DynamicImage> {
    image::load_from_memory_with_format(raw_png_data, image::ImageFormat::Png)
        .map_err(|e| e.to_string())
        .context("decoding PNG Image")
}

#[cfg(not(feature = "png-image"))]
pub fn decode(_: &[u8]) -> Result<DynamicImage> {
    bail!("Crate feature `png-image` is disabled; cannot decode PNG image");
}

#[cfg(feature = "png-image")]
pub fn encode(dyn_img: &DynamicImage) -> Result<Vec<u8>> {
    let mut png_data: Vec<u8> = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png_data);
    dyn_img
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| e.to_string())
        .context("encoding PNG Image")?;
    Ok(png_data)
}

#[cfg(not(feature = "png-image"))]
pub fn encode(_: &DynamicImage) -> Result<Vec<u8>> {
    bail!("Crate feature `png-image` is disabled; cannot encode PNG image");
}
