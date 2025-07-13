use std::borrow::Cow;
use std::convert::TryInto;
use image::{DynamicImage, ImageBuffer, Rgba};
use crate::gamemaker::printing::hexdump;

pub const MAX_CHUNK_SIZE: usize = 5;
pub const HEADER_SIZE: usize = 12;

const QOI_INDEX: u8 = 0x00;
const QOI_RUN_8: u8 = 0x40;
const QOI_RUN_16: u8 = 0x60;
const QOI_DIFF_8: u8 = 0x80;
const QOI_DIFF_16: u8 = 0xc0;
const QOI_DIFF_24: u8 = 0xe0;
const QOI_COLOR: u8 = 0xf0;
const QOI_MASK_2: u8 = 0xc0;
const QOI_MASK_3: u8 = 0xe0;
const QOI_MASK_4: u8 = 0xf0;


pub fn get_image_from_bytes(bytes: &[u8]) -> Result<DynamicImage, String> {
    let header: &[u8] = &bytes.get(..12).ok_or("Invalid QOI header (less than 12 bytes long)")?;
    
    let is_big_endian: bool = match &header[0..4] {
        b"qoif" => true,
        b"fioq" => false,
        _ => return Err(format!("Invalid QOIF image magic [{}]", hexdump(header, 0, Some(4))?))
    };
    
    let u32_from = if is_big_endian { u32::from_be_bytes } else { u32::from_le_bytes };
    let u16_from = if is_big_endian { u16::from_be_bytes } else { u16::from_le_bytes };

    let width: usize = u16_from(header[4..6].try_into().unwrap()) as usize;
    let height: usize = u16_from(header[6..8].try_into().unwrap()) as usize;
    let length: usize = u32_from(header[8..12].try_into().unwrap()) as usize;

    let pixel_data: &[u8] = &bytes.get(12..12+length).ok_or("Specified QOI data length out of bounds")?;

    let mut pos: usize = 0;
    let mut run: i32 = 0;
    let mut r: u8 = 0;
    let mut g: u8 = 0;
    let mut b: u8 = 0;
    let mut a: u8 = 255;
    let mut index: [u8; 256] = [0; 64*4];

    let mut img = ImageBuffer::new(width as u32, height as u32);
    for (_x, _y, pixel) in img.enumerate_pixels_mut() {
        if run > 0 {
            run -= 1;
            *pixel = Rgba([r, g, b, a]);
            continue
        }
        if pos >= pixel_data.len() {
            *pixel = Rgba([r, g, b, a]);
            continue
        }

        let b1: u8 = pixel_data[pos];
        pos += 1;

        if (b1 & QOI_MASK_2) == QOI_INDEX {
            let index_pos = ((b1 ^ QOI_INDEX) << 2) as usize;
            r = index[index_pos+0];
            g = index[index_pos+1];
            b = index[index_pos+2];
            a = index[index_pos+3];
        }
        else if (b1 & QOI_MASK_3) == QOI_RUN_8 {
            run = (b1 & 0x1F) as i32;
        }
        else if (b1 & QOI_MASK_3) == QOI_RUN_16 {
            let b2: u8 = pixel_data[pos];
            pos += 1;
            run = (((b1 & 0x1F) as i32) << 8 | b2 as i32) + 32;
        }
        else if (b1 & QOI_MASK_2) == QOI_DIFF_8 {
            r = r.wrapping_add(((b1 as i32 & 0x30) << 26 >> 30) as u8);
            g = g.wrapping_add(((b1 as i32 & 0x_C) << 28 >> 30) as u8);
            b = b.wrapping_add(((b1 as i32 & 0x_3) << 30 >> 30) as u8);
        }
        else if (b1 & QOI_MASK_3) == QOI_DIFF_16 {
            let b2: u8 = pixel_data[pos];
            pos += 1;
            let merged: i32 = (b1 as i32) << 8 | b2 as i32;
            r = r.wrapping_add(((merged & 0x1F00) << 19 >> 27) as u8);
            g = g.wrapping_add(((merged & 0x00F0) << 24 >> 28) as u8);
            b = b.wrapping_add(((merged & 0x000F) << 28 >> 28) as u8);
        }
        else if (b1 & QOI_MASK_4) == QOI_DIFF_24 {
            let b2: i32 = pixel_data[pos] as i32;
            let b3: i32 = pixel_data[pos + 1] as i32;
            pos += 2;
            let merged: i32 = ((b1 as i32) << 16) | (b2 << 8) | b3;
            r = r.wrapping_add(((merged & 0x_F8000) << 12 >> 27) as u8);
            g = g.wrapping_add(((merged & 0x__7C00) << 17 >> 27) as u8);
            b = b.wrapping_add(((merged & 0x___3E0) << 22 >> 27) as u8);
            a = a.wrapping_add(((merged & 0x____1F) << 27 >> 27) as u8);
        }
        else if (b1 & QOI_MASK_4) == QOI_COLOR {
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
            return Err(format!("Invalid QOI opcode 0x{b1}"))
        }

        let index_pos: usize = (((r ^ g ^ b ^ a) & 0x3F) << 2) as usize;
        index[index_pos+0] = r;
        index[index_pos+1] = g;
        index[index_pos+2] = b;
        index[index_pos+3] = a;
        *pixel = Rgba([r, g, b, a]);
    }

    Ok(DynamicImage::ImageRgba8(img))
}


/// TODO: this function is broken; fix it
pub fn get_bytes_from_image(img: &DynamicImage) -> Vec<u8> {
    let width: usize = img.width() as usize;
    let height: usize = img.height() as usize;
    let required_size: usize = (width * height * MAX_CHUNK_SIZE) + HEADER_SIZE;
    let mut buffer: Vec<u8> = vec![0; required_size];

    // Little-endian QOIF image magic
    buffer[0] = b'f';
    buffer[1] = b'i';
    buffer[2] = b'o';
    buffer[3] = b'q';
    // TODO: support big endian
    buffer[4..6].copy_from_slice(&(width as u16).to_le_bytes());
    buffer[6..8].copy_from_slice(&(height as u16).to_le_bytes());

    let rgba_image = match img {
        DynamicImage::ImageRgba8(i) => Cow::Borrowed(i),
        _ => Cow::Owned(img.to_rgba8()),
    };
    let raw_data: &Vec<u8> = rgba_image.as_ref().as_raw();
    let raw_data_length: usize = raw_data.len();
    let mut res_pos: usize = HEADER_SIZE;

    let mut r: u8;
    let mut g: u8;
    let mut b: u8;
    let mut a: u8;
    let mut run: i32 = 0;
    let mut v: u32;
    let mut v_prev: u32 = 0xff;
    let mut index: [u32; 64] = [0; 64];

    for raw_data_pos in (0..raw_data_length).step_by(4) {
        r = raw_data[raw_data_pos + 0];
        g = raw_data[raw_data_pos + 1];
        b = raw_data[raw_data_pos + 2];
        a = raw_data[raw_data_pos + 3];

        v = ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (a as u32);
        if v == v_prev {
            run += 1;
        }
        if run > 0 && (run == 0x2020 || v != v_prev || raw_data_pos == raw_data_length - 4) {
            if run < 33 {
                run -= 1;
                buffer[res_pos] = QOI_RUN_8 | (run as u8);
                res_pos += 1;
            } else {
                run -= 33;
                buffer[res_pos] = QOI_RUN_16 | ((run >> 8) as u8);
                buffer[res_pos + 1] = (run & 0xff) as u8;
                res_pos += 2;
            }
            run = 0;
        }
        if v != v_prev {
            let index_pos = ((r ^ g ^ b ^ a) & 0x3f) as usize;
            if index[index_pos] == v {
                buffer[res_pos] = QOI_INDEX | (index_pos as u8);
                res_pos += 1;
            } else {
                index[index_pos] = v;

                let vr = r.wrapping_sub((v_prev >> 24) as u8) as i16;
                let vg = g.wrapping_sub((v_prev >> 16) as u8) as i16;
                let vb = b.wrapping_sub((v_prev >> 8) as u8) as i16;
                let va = a.wrapping_sub(v_prev as u8) as i16;

                if (-16..16).contains(&vr)
                    && (-16..16).contains(&vg)
                    && (-16..16).contains(&vb)
                    && (-16..16).contains(&va)
                {
                    if va == 0
                        && (-2..2).contains(&vr)
                        && (-2..2).contains(&vg)
                        && (-2..2).contains(&vb)
                    {
                        buffer[res_pos] = QOI_DIFF_8 | ((vr as u8) << 4 & 0x30) | ((vg as u8) << 2 & 0x0c) | (vb as u8 & 0x03);
                        res_pos += 1;
                    } else if va == 0
                        && (-8..8).contains(&vg)
                        && (-8..8).contains(&vb)
                    {
                        buffer[res_pos] = QOI_DIFF_16 | (vr as u8 & 0x1f);
                        buffer[res_pos + 1] = ((vg as u8) << 4 & 0xf0) | (vb as u8 & 0x0f);
                        res_pos += 2;
                    } else {
                        buffer[res_pos] = QOI_DIFF_24 | ((vr >> 1) as u8 & 0x0f);
                        buffer[res_pos + 1] = ((vr as u8) << 7 & 0x80) | ((vg as u8) << 2 & 0x7c) | ((vb >> 3) as u8 & 0x03);
                        buffer[res_pos + 2] = ((vb as u8) << 5 & 0xe0) | (va as u8 & 0x1f);
                        res_pos += 3;
                    }
                } else {
                    let mut flags = 0u8;
                    if r != (v_prev >> 24) as u8 { flags |= 0x08; }
                    if g != (v_prev >> 16) as u8 { flags |= 0x04; }
                    if b != (v_prev >> 8) as u8 { flags |= 0x02; }
                    if a != v_prev as u8 { flags |= 0x01; }

                    buffer[res_pos] = QOI_COLOR | flags;
                    res_pos += 1;
                    if flags & 0x08 != 0 { buffer[res_pos] = r; res_pos += 1; }
                    if flags & 0x04 != 0 { buffer[res_pos] = g; res_pos += 1; }
                    if flags & 0x02 != 0 { buffer[res_pos] = b; res_pos += 1; }
                    if flags & 0x01 != 0 { buffer[res_pos] = a; res_pos += 1; }
                }
            }
        }

        v_prev = v;
    }

    let length = (res_pos - HEADER_SIZE) as u32;
    // TODO: support big endian
    buffer[8..12].copy_from_slice(&length.to_le_bytes());

    buffer.truncate(res_pos);
    buffer
}


