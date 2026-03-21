//! An implementation of GameMaker's slightly custom QOI ("Quite Ok Image") image format.

use std::{borrow::Cow, convert::TryInto};

use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};

use crate::{
    prelude::*,
    util::fmt::hexdump_range,
    wad::{data::Endianness, serialize::builder::DataBuilder},
};

const QOI_INDEX: u8 = 0x00;
const QOI_RUN_8: u8 = 0x40;
const QOI_RUN_16: u8 = 0x60;
const QOI_DIFF_8: u8 = 0x80;
const QOI_DIFF_16: u8 = 0xC0;
const QOI_DIFF_24: u8 = 0xE0;
const QOI_COLOR: u8 = 0xF0;
const QOI_MASK_2: u8 = 0xC0;
const QOI_MASK_3: u8 = 0xE0;
const QOI_MASK_4: u8 = 0xF0;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct Pixel {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Pixel {
    const DEFAULT: Self = Self::new(0, 0, 0, 255);

    #[must_use]
    const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    #[must_use]
    const fn from_array(channels: [u8; 4]) -> Self {
        Self::new(channels[0], channels[1], channels[2], channels[3])
    }

    #[must_use]
    const fn to_array(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PixelDiff {
    r: i16,
    g: i16,
    b: i16,
    a: i16,
}

impl PixelDiff {
    #[must_use]
    fn new(current: Pixel, previous: Pixel) -> Self {
        Self {
            r: i16::from(current.r) - i16::from(previous.r),
            g: i16::from(current.g) - i16::from(previous.g),
            b: i16::from(current.b) - i16::from(previous.b),
            a: i16::from(current.a) - i16::from(previous.a),
        }
    }

    #[must_use]
    fn fits(channel: i16, max_val: u8) -> bool {
        let max_val = i16::from(max_val);
        channel > -max_val && channel < max_val
    }

    #[must_use]
    fn fits_r(&self, max_val: u8) -> bool {
        Self::fits(self.r, max_val)
    }
    #[must_use]
    fn fits_g(&self, max_val: u8) -> bool {
        Self::fits(self.g, max_val)
    }
    #[must_use]
    fn fits_b(&self, max_val: u8) -> bool {
        Self::fits(self.b, max_val)
    }
    #[must_use]
    fn fits_a(&self, max_val: u8) -> bool {
        Self::fits(self.a, max_val)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QoiHeader {
    pub width: u16,
    pub height: u16,
    pub length: u32,
}

fn estimate_encoded_size(image: &DynamicImage) -> usize {
    let width = image.width() as usize;
    let height = image.height() as usize;
    // apparently not multiplying by anything is the best estimate???
    width * height
}

pub fn build(image: &DynamicImage, builder: &mut DataBuilder) -> Result<()> {
    builder.raw_data.reserve(estimate_encoded_size(image));
    encode_to_buffer(image, &mut builder.raw_data)
}

pub fn encode(image: &DynamicImage) -> Result<Vec<u8>> {
    let mut buffer = Vec::with_capacity(estimate_encoded_size(image));
    encode_to_buffer(image, &mut buffer)?;
    Ok(buffer)
}

// QOI implementations

pub fn read_header(bytes: &[u8]) -> Result<QoiHeader> {
    // idk if this endianness thing works, it's untested.
    let header: &[u8] = bytes
        .get(0..12)
        .ok_or("Invalid QOI header (less than 12 bytes long)")?;

    let endianness: Endianness = match &header[0..4] {
        b"qoif" => Endianness::Big,
        b"fioq" => Endianness::Little,
        _ => bail!("Invalid QOI image magic [{}]", hexdump_range(header, 0..4)?),
    };

    let u16_from = match endianness {
        Endianness::Little => u16::from_le_bytes,
        Endianness::Big => u16::from_be_bytes,
    };
    let u32_from = match endianness {
        Endianness::Little => u32::from_le_bytes,
        Endianness::Big => u32::from_be_bytes,
    };

    let width = u16_from(header[4..6].try_into().unwrap());
    let height = u16_from(header[6..8].try_into().unwrap());
    let length = u32_from(header[8..12].try_into().unwrap());

    Ok(QoiHeader { width, height, length })
}

pub fn decode(bytes: &[u8]) -> Result<DynamicImage> {
    let header: QoiHeader = read_header(bytes).context("reading QOI header")?;

    let pixel_data: &[u8] = bytes
        .get(12..12 + header.length as usize)
        .ok_or("Specified QOI data length out of bounds")?;

    let mut pos: usize = 0;
    let mut run: i32 = 0;
    let mut px = Pixel::DEFAULT;
    let mut index = [Pixel::DEFAULT; 64];

    let width = u32::from(header.width);
    let height = u32::from(header.height);
    let mut image: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(width, height);

    for (_x, _y, pixel) in image.enumerate_pixels_mut() {
        if run > 0 {
            run -= 1;
            pixel.0 = px.to_array();
            continue;
        }
        if pos >= pixel_data.len() {
            pixel.0 = px.to_array();
            continue;
        }

        let b1: u8 = pixel_data[pos];
        pos += 1;

        if (b1 & QOI_MASK_2) == QOI_INDEX {
            let index_pos = (b1 ^ QOI_INDEX) as usize;
            px = index[index_pos];
        } else if (b1 & QOI_MASK_3) == QOI_RUN_8 {
            run = i32::from(b1 & 0x1F);
        } else if (b1 & QOI_MASK_3) == QOI_RUN_16 {
            let b2: u8 = pixel_data[pos];
            pos += 1;
            run = (i32::from(b1 & 0x1F) << 8 | i32::from(b2)) + 32;
        } else if (b1 & QOI_MASK_2) == QOI_DIFF_8 {
            px.r =
                px.r.wrapping_add(((i32::from(b1) & 0x30) << 26 >> 30) as u8);
            px.g =
                px.g.wrapping_add(((i32::from(b1) & 0x0C) << 28 >> 30) as u8);
            px.b =
                px.b.wrapping_add(((i32::from(b1) & 0x_3) << 30 >> 30) as u8);
        } else if (b1 & QOI_MASK_3) == QOI_DIFF_16 {
            let b2: u8 = pixel_data[pos];
            pos += 1;
            let merged: i32 = i32::from(b1) << 8 | i32::from(b2);
            px.r = px.r.wrapping_add(((merged & 0x1F00) << 19 >> 27) as u8);
            px.g = px.g.wrapping_add(((merged & 0x00F0) << 24 >> 28) as u8);
            px.b = px.b.wrapping_add(((merged & 0x000F) << 28 >> 28) as u8);
        } else if (b1 & QOI_MASK_4) == QOI_DIFF_24 {
            let b2: i32 = i32::from(pixel_data[pos]);
            let b3: i32 = i32::from(pixel_data[pos + 1]);
            pos += 2;
            let merged: i32 = (i32::from(b1) << 16) | (b2 << 8) | b3;
            px.r = px.r.wrapping_add(((merged & 0x0F_8000) << 12 >> 27) as u8);
            px.g = px.g.wrapping_add(((merged & 0x00_7C00) << 17 >> 27) as u8);
            px.b = px.b.wrapping_add(((merged & 0x00_03E0) << 22 >> 27) as u8);
            px.a = px.a.wrapping_add(((merged & 0x00_001F) << 27 >> 27) as u8);
        } else if (b1 & QOI_MASK_4) == QOI_COLOR {
            if (b1 & 8) != 0 {
                px.r = pixel_data[pos];
                pos += 1;
            }
            if (b1 & 4) != 0 {
                px.g = pixel_data[pos];
                pos += 1;
            }
            if (b1 & 2) != 0 {
                px.b = pixel_data[pos];
                pos += 1;
            }
            if (b1 & 1) != 0 {
                px.a = pixel_data[pos];
                pos += 1;
            }
        } else {
            bail!("Invalid QOI Opcode {b1} (0x{b1:02X})");
        }

        let index_pos = ((px.r ^ px.g ^ px.b ^ px.a) & 0x3F) as usize;
        index[index_pos] = px;
        pixel.0 = px.to_array();
    }

    Ok(DynamicImage::ImageRgba8(image))
}

fn encode_to_buffer(image: &DynamicImage, buffer: &mut Vec<u8>) -> Result<()> {
    // (big endian unsupported)
    let (width, height) = image.dimensions();
    let width = u16::try_from(width).context_src("Image width exceeds limit of 65535")?;
    let height = u16::try_from(height).context_src("Image height exceeds limit of 65535")?;
    let image = image
        .as_rgba8()
        .map_or_else(|| Cow::Owned(image.to_rgba8()), Cow::Borrowed);
    let mut image_pixels = image.pixels().peekable();

    // Write header (12 bytes)
    let start_pos: usize = buffer.len();
    buffer.extend_from_slice(b"fioq");
    buffer.extend_from_slice(&width.to_le_bytes());
    buffer.extend_from_slice(&height.to_le_bytes());
    buffer.extend_from_slice(&[0u8; 4]); // placeholder for length

    // Prepare some vars
    let mut run: i32 = 0;
    let mut index = [Pixel::DEFAULT; 64];
    let mut px_prev = Pixel::DEFAULT;

    // Start with QOI looping
    while let Some(&px) = image_pixels.next() {
        let px = Pixel::from_array(px.0);

        let is_last: bool = image_pixels.peek().is_none();
        let is_same: bool = px == px_prev;

        if is_same {
            run += 1;
        }

        if run > 0 && (run == 0x2020 || !is_same || is_last) {
            if run < 33 {
                run -= 1;
                buffer.push(QOI_RUN_8 | run as u8);
            } else {
                run -= 33;
                buffer.push(QOI_RUN_16 | (run >> 8) as u8);
                buffer.push(run as u8);
            }
            run = 0;
        }

        // same pixel lol, run value will handle it
        if is_same {
            px_prev = px;
            continue;
        }

        // if cached in index, easy.
        let index_pos: u8 = (px.r ^ px.g ^ px.b ^ px.a) & 63;
        if index[index_pos as usize] == px {
            buffer.push(QOI_INDEX | index_pos);
            px_prev = px;
            continue;
        }

        index[index_pos as usize] = px;

        let diff = PixelDiff::new(px, px_prev);

        if diff.fits_r(16) && diff.fits_g(16) & diff.fits_b(16) && diff.fits_a(16) {
            if diff.a == 0 && diff.fits_r(2) && diff.fits_g(2) && diff.fits_b(2) {
                buffer.push(
                    (i16::from(QOI_DIFF_8)
                        | (diff.r << 4 & 0x30)
                        | (diff.g << 2 & 0x0C)
                        | (diff.b & 3)) as u8,
                );
            } else if diff.a == 0 && diff.fits_g(8) && diff.fits_b(8) {
                buffer.push((i16::from(QOI_DIFF_16) | (diff.r & 31)) as u8);
                buffer.push(((diff.g << 4 & 0xF0) | (diff.b & 15)) as u8);
            } else {
                buffer.push((i16::from(QOI_DIFF_24) | (diff.r >> 1 & 15)) as u8);
                buffer
                    .push(((diff.r << 7 & 0x80) | (diff.g << 2 & 0x7C) | (diff.b >> 3 & 3)) as u8);
                buffer.push(((diff.b << 5 & 0xE0) | (diff.a & 31)) as u8);
            }
        } else {
            let mut mask = 0;
            if diff.r != 0 {
                mask |= 8;
            }
            if diff.g != 0 {
                mask |= 4;
            }
            if diff.b != 0 {
                mask |= 2;
            }
            if diff.a != 0 {
                mask |= 1;
            }
            buffer.push(QOI_COLOR | mask);
            if diff.r != 0 {
                buffer.push(px.r);
            }
            if diff.g != 0 {
                buffer.push(px.g);
            }
            if diff.b != 0 {
                buffer.push(px.b);
            }
            if diff.a != 0 {
                buffer.push(px.a);
            }
        }

        px_prev = px;
    }

    let length: usize = buffer.len() - start_pos - 12;
    let length_encoded = (length as u32).to_le_bytes();
    let range = (start_pos + 8)..(start_pos + 12);
    buffer[range].copy_from_slice(&length_encoded);
    Ok(())
}
