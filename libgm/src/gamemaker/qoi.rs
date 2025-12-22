use std::convert::TryInto;

use image::{DynamicImage, ImageBuffer, Rgba};

use crate::{gamemaker::data::Endianness, prelude::*, util::fmt::hexdump_range};

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

pub fn deserialize(bytes: &[u8]) -> Result<DynamicImage> {
    let header: &[u8] = bytes
        .get(..12)
        .ok_or("Invalid QOI header (less than 12 bytes long)")?;

    let endianness: Endianness = match &header[..4] {
        b"qoif" => Endianness::Big,
        b"fioq" => Endianness::Little,
        _ => bail!("Invalid QOIF image magic [{}]", hexdump_range(header, ..4)?),
    };

    let u16_from = match endianness {
        Endianness::Little => u16::from_le_bytes,
        Endianness::Big => u16::from_be_bytes,
    };
    let u32_from = match endianness {
        Endianness::Little => u32::from_le_bytes,
        Endianness::Big => u32::from_be_bytes,
    };

    let width = u32::from(u16_from(header[4..6].try_into().unwrap()));
    let height = u32::from(u16_from(header[6..8].try_into().unwrap()));
    let length = u32_from(header[8..12].try_into().unwrap()) as usize;

    let pixel_data: &[u8] = bytes
        .get(12..12 + length)
        .ok_or("Specified QOI data length out of bounds")?;

    let mut pos: usize = 0;
    let mut run: i32 = 0;
    let mut r: u8 = 0;
    let mut g: u8 = 0;
    let mut b: u8 = 0;
    let mut a: u8 = 255;
    let mut index: [u8; 256] = [0; 64 * 4];

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
            bail!("Invalid QOI opcode {b1} (0x{b1:02X})");
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
