//! An implementation of GameMaker's custom QOI ("Quite Ok Image") image format.

use std::{borrow::Cow, convert::TryInto};

use image::{DynamicImage, ImageBuffer, Rgba};

use crate::{
    gamemaker::{data::Endianness, serialize::builder::DataBuilder},
    prelude::*,
    util::fmt::hexdump_range,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QoiHeader {
    pub width: u16,
    pub height: u16,
    pub length: u32,
}

pub fn decode(bytes: &[u8]) -> Result<DynamicImage> {
    decode_(bytes).context("decoding QOI image")
}

pub fn build(image: &DynamicImage, builder: &mut DataBuilder) -> Result<()> {
    encode_(image, &mut builder.raw_data).context("encoding QOI image")
}

pub fn encode(image: &DynamicImage) -> Result<Vec<u8>> {
    /// Maximum chunk size according to the QOI spec <https://qoiformat.org/qoi-specification.pdf>
    const MAX_CHUNK_SIZE: usize = 5;

    let width = image.width() as usize;
    let height = image.height() as usize;
    let cap = width * height * MAX_CHUNK_SIZE;
    let mut buffer = Vec::with_capacity(cap);
    encode_(image, &mut buffer).context("encoding QOI image")?;
    Ok(buffer)
}

// QOI implementations

pub fn read_header(bytes: &[u8]) -> Result<QoiHeader> {
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

fn decode_(bytes: &[u8]) -> Result<DynamicImage> {
    let header: QoiHeader = read_header(bytes).context("reading QOI header")?;

    let pixel_data: &[u8] = bytes
        .get(12..12 + header.length as usize)
        .ok_or("Specified QOI data length out of bounds")?;

    let mut pos: usize = 0;
    let mut run: i32 = 0;
    let mut r: u8 = 0;
    let mut g: u8 = 0;
    let mut b: u8 = 0;
    let mut a: u8 = 255;
    let mut index: [u8; 256] = [0; 64 * 4];

    let width = u32::from(header.width);
    let height = u32::from(header.height);
    let mut img = ImageBuffer::new(width, height);

    for (_x, _y, pixel) in img.enumerate_pixels_mut() {
        if run > 0 {
            run -= 1;
            *pixel = Rgba([r, g, b, a]);
            continue;
        }
        if pos >= pixel_data.len() {
            *pixel = Rgba([r, g, b, a]);
            continue;
        }

        let b1: u8 = pixel_data[pos];
        pos += 1;

        if (b1 & QOI_MASK_2) == QOI_INDEX {
            let index_pos = ((b1 ^ QOI_INDEX) << 2) as usize;
            r = index[index_pos];
            g = index[index_pos + 1];
            b = index[index_pos + 2];
            a = index[index_pos + 3];
        } else if (b1 & QOI_MASK_3) == QOI_RUN_8 {
            run = i32::from(b1 & 0x1F);
        } else if (b1 & QOI_MASK_3) == QOI_RUN_16 {
            let b2: u8 = pixel_data[pos];
            pos += 1;
            run = (i32::from(b1 & 0x1F) << 8 | i32::from(b2)) + 32;
        } else if (b1 & QOI_MASK_2) == QOI_DIFF_8 {
            r = r.wrapping_add(((i32::from(b1) & 0x30) << 26 >> 30) as u8);
            g = g.wrapping_add(((i32::from(b1) & 0x0C) << 28 >> 30) as u8);
            b = b.wrapping_add(((i32::from(b1) & 0x_3) << 30 >> 30) as u8);
        } else if (b1 & QOI_MASK_3) == QOI_DIFF_16 {
            let b2: u8 = pixel_data[pos];
            pos += 1;
            let merged: i32 = i32::from(b1) << 8 | i32::from(b2);
            r = r.wrapping_add(((merged & 0x1F00) << 19 >> 27) as u8);
            g = g.wrapping_add(((merged & 0x00F0) << 24 >> 28) as u8);
            b = b.wrapping_add(((merged & 0x000F) << 28 >> 28) as u8);
        } else if (b1 & QOI_MASK_4) == QOI_DIFF_24 {
            let b2: i32 = i32::from(pixel_data[pos]);
            let b3: i32 = i32::from(pixel_data[pos + 1]);
            pos += 2;
            let merged: i32 = (i32::from(b1) << 16) | (b2 << 8) | b3;
            r = r.wrapping_add(((merged & 0x0F_8000) << 12 >> 27) as u8);
            g = g.wrapping_add(((merged & 0x00_7C00) << 17 >> 27) as u8);
            b = b.wrapping_add(((merged & 0x00_03E0) << 22 >> 27) as u8);
            a = a.wrapping_add(((merged & 0x00_001F) << 27 >> 27) as u8);
        } else if (b1 & QOI_MASK_4) == QOI_COLOR {
            if (b1 & 8) != 0 {
                r = pixel_data[pos];
                pos += 1;
            }
            if (b1 & 4) != 0 {
                g = pixel_data[pos];
                pos += 1;
            }
            if (b1 & 2) != 0 {
                b = pixel_data[pos];
                pos += 1;
            }
            if (b1 & 1) != 0 {
                a = pixel_data[pos];
                pos += 1;
            }
        } else {
            bail!("Invalid QOI Opcode {b1} (0x{b1:02X})");
        }

        let index_pos = (((r ^ g ^ b ^ a) & 0x3F) << 2) as usize;
        index[index_pos] = r;
        index[index_pos + 1] = g;
        index[index_pos + 2] = b;
        index[index_pos + 3] = a;
        *pixel = Rgba([r, g, b, a]);
    }

    Ok(DynamicImage::ImageRgba8(img))
}

#[allow(clippy::many_single_char_names)] // go fuck urself clippy
fn encode_(image: &DynamicImage, buffer: &mut Vec<u8>) -> Result<()> {
    let width = image.width() as u16;
    let height = image.height() as u16;
    let image = image
        .as_rgba8()
        .map_or_else(|| Cow::Owned(image.to_rgba8()), Cow::Borrowed);
    let image_bytes = image.as_raw();

    // Write header (12 bytes)
    let start_pos: usize = buffer.len();
    buffer.extend_from_slice(b"fioq");
    buffer.extend_from_slice(&width.to_le_bytes());
    buffer.extend_from_slice(&height.to_le_bytes());
    buffer.extend_from_slice(&[0u8; 4]); // placeholder for length

    // Prepare some vars
    let mut run: i32 = 0;
    let mut index = [[0u8; 4]; 64];
    let mut px_prev: [u8; 4] = [0, 0, 0, 255];
    let mut it = image_bytes.windows(4).peekable();

    // Start with QOI looping
    while let Some(px) = it.next() {
        let px: [u8; 4] = px.try_into().unwrap();
        let r: u8 = px[0];
        let g: u8 = px[1];
        let b: u8 = px[2];
        let a: u8 = px[3];

        let is_last: bool = it.next().is_none();
        let is_same: bool = px == px_prev;
        px_prev = px;

        if is_same {
            run += 1;
        }

        if run > 0 && (run == 0x2020 || !is_same || is_last) {
            if run < 33 {
                run -= 1;
                buffer.push(QOI_RUN_8 | (run) as u8);
            } else {
                run -= 33;
                buffer.push(QOI_RUN_16 | (run >> 8) as u8);
                buffer.push(run as u8);
            }
            run = 0;
        }

        if !is_same {
            let index_pos: u8 = (r ^ g ^ b ^ a) & 63;
            if index[index_pos as usize] == px {
                buffer.push(QOI_INDEX | index_pos);
            } else {
                index[index_pos as usize] = px;
            }

            let v: [i16; 4] = pixel_diff(px, px_prev);
            let vr = v[0] as u8;
            let vg = v[1] as u8;
            let vb = v[2] as u8;
            let va = v[3] as u8;

            // if all channels (r, g, b, a) are in (-17, 16)
            if v.iter().all(|&x| x > -17 && x < 16) {
                // if alpha is zero and r, g, b are in (-3, 2)
                if va == 0 && v[..3].iter().all(|&x| x > -3 && x < 2) {
                    buffer.push(QOI_DIFF_8 | (vr << 4 & 48) | (vg << 2 & 12) | (vb & 3));
                }
                // if alpha is zero and g, b are in (-9, 8)
                else if va == 0 && v[1..3].iter().all(|&x| x > -9 && x < 8) {
                    buffer.push(QOI_DIFF_16 | (vr & 31));
                    buffer.push((vg << 4 & 240) | (vb & 15));
                } else {
                    buffer.push(QOI_DIFF_24 | (vr >> 1 & 15));
                    buffer.push((vr << 7 & 128) | (vg << 2 & 124) | (vb >> 3 & 3));
                    buffer.push((vb << 5 & 224) | (va & 31));
                }
            } else {
                let mut mask = 0;
                if vr != 0 {
                    mask |= 8;
                }
                if vg != 0 {
                    mask |= 4;
                }
                if vb != 0 {
                    mask |= 2;
                }
                if va != 0 {
                    mask |= 1;
                }
                buffer.push(QOI_COLOR | mask);
                if vr != 0 {
                    buffer.push(r);
                }
                if vg != 0 {
                    buffer.push(g);
                }
                if vb != 0 {
                    buffer.push(b);
                }
                if va != 0 {
                    buffer.push(a);
                }
            }
        }
    }

    let length: usize = buffer.len() - start_pos - 12;
    let range = start_pos + 8..start_pos + 12;
    buffer[range].copy_from_slice(&(length as u32).to_le_bytes());

    Ok(())
}

fn pixel_diff(curr: [u8; 4], prev: [u8; 4]) -> [i16; 4] {
    let mut diffs = [0i16; 4];
    for i in 0..4 {
        diffs[i] = i16::from(curr[i]) - i16::from(prev[i]);
    }
    diffs
}
