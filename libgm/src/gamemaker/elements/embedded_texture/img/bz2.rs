use image::DynamicImage;

use super::qoi;
use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct BZip2QoiHeader {
    pub width: u16,
    pub height: u16,
    /// Present in 2022.5+
    pub uncompressed_size: Option<u32>,
}

impl BZip2QoiHeader {
    #[must_use]
    pub const fn new(width: u16, height: u16, uncompressed_size: Option<u32>) -> Self {
        Self { width, height, uncompressed_size }
    }
}

#[cfg(feature = "bzip2-image")]
pub fn decompress(compressed_bzip2_data: &[u8]) -> Result<Vec<u8>> {
    use std::io::Read;
    let mut decoder = bzip2::read::BzDecoder::new(compressed_bzip2_data);
    let mut decompressed_data: Vec<u8> = Vec::new();
    decoder
        .read_to_end(&mut decompressed_data)
        .map_err(|e| e.to_string())
        .context("decoding BZip2 stream")?;
    Ok(decompressed_data)
}

#[cfg(not(feature = "bzip2-image"))]
pub fn decompress(_: &[u8]) -> Result<Vec<u8>> {
    bail!("Crate feature `bzip2-images` is disabled; cannot decode BZip2 stream");
}

#[cfg(feature = "bzip2-image")]
pub fn compress(data: &[u8]) -> Result<Vec<u8>> {
    use std::io::Read;

    // At some point, allow library users to customize this.
    let level = bzip2::Compression::fast();

    let mut encoder = bzip2::read::BzEncoder::new(data, level);
    let mut compressed_data: Vec<u8> = Vec::new();
    encoder
        .read_to_end(&mut compressed_data)
        .map_err(|e| e.to_string())
        .context("decoding BZip2 stream")?;
    Ok(compressed_data)
}

#[cfg(not(feature = "bzip2-image"))]
pub fn compress(_: &[u8]) -> Result<Vec<u8>> {
    bail!("Crate feature `bzip2-images` is disabled; cannot encode BZip2 stream");
}

// Bz2Qoi functions

pub fn decode_image(raw_bz2_qoi_data: &[u8]) -> Result<DynamicImage> {
    let decompressed_data: Vec<u8> = decompress(raw_bz2_qoi_data)?;
    let image: DynamicImage = qoi::decode(&decompressed_data)?;
    Ok(image)
}

pub fn encode_image(dyn_img: &DynamicImage) -> Result<(Vec<u8>, BZip2QoiHeader)> {
    let qoi_data: Vec<u8> = qoi::encode(dyn_img)?;
    let header = BZip2QoiHeader {
        width: dyn_img.width() as u16,
        height: dyn_img.height() as u16,
        uncompressed_size: Some(qoi_data.len() as u32),
    };
    let bz2_data: Vec<u8> = compress(&qoi_data)?;
    Ok((bz2_data, header))
}
