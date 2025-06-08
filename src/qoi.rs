use std::borrow::Cow;
use std::io::Read;
use std::convert::TryInto;
use image::{DynamicImage, ImageBuffer, Rgba};


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

pub fn get_image_from_stream<R: Read>(mut stream: R) -> Result<DynamicImage, String> {
    let mut header = [0u8; 12];
    stream.read_exact(&mut header).map_err(|e| e.to_string())?;

    let length = u32::from_le_bytes(header[8..12].try_into().unwrap()) as usize;
    let mut bytes = vec![0u8; 12 + length];
    stream.read_exact(&mut bytes[12..]).map_err(|e| e.to_string())?;
    bytes[..12].copy_from_slice(&header);

    get_image_from_bytes(&bytes)
}

pub fn get_image_from_bytes(bytes: &[u8]) -> Result<DynamicImage, String> {
    if bytes.len() < 12 {
        return Err("Invalid QOI header".to_string());
    }

    let header = &bytes[..12];
    if header[0] != b'f' || header[1] != b'i' || header[2] != b'o' || header[3] != b'q' {
        return Err("Invalid little-endian QOIF image magic".to_string());
    }

    let width = u16::from_le_bytes(header[4..6].try_into().unwrap()) as usize;
    let height = u16::from_le_bytes(header[6..8].try_into().unwrap()) as usize;
    let length = u32::from_le_bytes(header[8..12].try_into().unwrap()) as usize;

    if bytes.len() < 12 + length {
        return Err("Invalid QOI data length".to_string());
    }

    let pixel_data = &bytes[12..12+length];

    let mut pos = 0;
    let mut run = 0;
    let mut r = 0u8;
    let mut g = 0u8;
    let mut b = 0u8;
    let mut a = 255u8;
    let mut index = [[0u8; 4]; 64];

    let mut img = ImageBuffer::new(width as u32, height as u32);
    for (_x, _y, pixel) in img.enumerate_pixels_mut() {
        if run > 0 {
            run -= 1;
        } else if pos < pixel_data.len() {
            let b1 = pixel_data[pos];
            pos += 1;

            if (b1 & QOI_MASK_2) == QOI_INDEX {
                let index_pos = (b1 ^ QOI_INDEX) as usize;
                r = index[index_pos][0];
                g = index[index_pos][1];
                b = index[index_pos][2];
                a = index[index_pos][3];
            } else if (b1 & QOI_MASK_3) == QOI_RUN_8 {
                run = (b1 & 0x1f) as usize;
            } else if (b1 & QOI_MASK_3) == QOI_RUN_16 {
                let b2 = pixel_data[pos] as usize;
                pos += 1;
                run = (((b1 & 0x1f) as usize) << 8 | b2) + 32;
            } else if (b1 & QOI_MASK_2) == QOI_DIFF_8 {
                r = r.wrapping_add(((b1 & 0x30) >> 4).wrapping_sub(2));
                g = g.wrapping_add(((b1 & 0x0c) >> 2).wrapping_sub(2));
                b = b.wrapping_add((b1 & 0x03).wrapping_sub(2));
            } else if (b1 & QOI_MASK_3) == QOI_DIFF_16 {
                let b2 = pixel_data[pos];
                pos += 1;
                let merged = ((b1 as u16) << 8) | (b2 as u16);
                r = r.wrapping_add(((merged & 0x1f00) >> 8) as u8);
                g = g.wrapping_add((((merged & 0x00f0) >> 4) as u8).wrapping_sub(8));
                b = b.wrapping_add(((merged & 0x000f) as u8).wrapping_sub(8));
            } else if (b1 & QOI_MASK_4) == QOI_DIFF_24 {
                let b2 = pixel_data[pos];
                pos += 1;
                let b3 = pixel_data[pos];
                pos += 1;
                let merged = ((b1 as u32) << 16) | ((b2 as u32) << 8) | (b3 as u32);
                r = r.wrapping_add(((merged & 0x0f0000) >> 16) as u8);
                g = g.wrapping_add((((merged & 0x00f800) >> 11) as u8).wrapping_sub(16));
                b = b.wrapping_add((((merged & 0x0007c0) >> 6) as u8).wrapping_sub(16));
                a = a.wrapping_add(((merged & 0x00003f) as u8).wrapping_sub(32));
            } else if (b1 & QOI_MASK_4) == QOI_COLOR {
                if (b1 & 0x08) != 0 {
                    r = pixel_data[pos];
                    pos += 1;
                }
                if (b1 & 0x04) != 0 {
                    g = pixel_data[pos];
                    pos += 1;
                }
                if (b1 & 0x02) != 0 {
                    b = pixel_data[pos];
                    pos += 1;
                }
                if (b1 & 0x01) != 0 {
                    a = pixel_data[pos];
                    pos += 1;
                }
            }

            let index_pos = ((r ^ g ^ b ^ a) & 0x3f) as usize;
            index[index_pos] = [r, g, b, a];
        }
        
        *pixel = Rgba([r, g, b, a]);
    }

    Ok(DynamicImage::ImageRgba8(img))
}

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
    let mut v_prev: u32 = 0xffu32;
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
                        buffer[res_pos] = QOI_DIFF_8
                            | (((vr + 2) as u8) << 4 & 0x30)
                            | (((vg + 2) as u8) << 2 & 0x0c)
                            | ((vb + 2) as u8 & 0x03);
                        res_pos += 1;
                    } else if va == 0
                        && (-8..8).contains(&vg)
                        && (-8..8).contains(&vb)
                    {
                        buffer[res_pos] = QOI_DIFF_16 | ((vr + 16) as u8 & 0x1f);
                        buffer[res_pos + 1] = (((vg + 8) as u8) << 4 & 0xf0) | ((vb + 8) as u8 & 0x0f);
                        res_pos += 2;
                    } else {
                        buffer[res_pos] = QOI_DIFF_24 | (((vr + 16) >> 1) as u8 & 0x0f);
                        buffer[res_pos + 1] = (((vr + 16) as u8) << 7 & 0x80)
                            | (((vg + 32) as u8) << 2 & 0x7c)
                            | (((vb + 32) >> 3) as u8 & 0x03);
                        buffer[res_pos + 2] = (((vb + 32) as u8) << 5 & 0xe0) | ((va + 32) as u8 & 0x1f);
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
    buffer[8..12].copy_from_slice(&length.to_le_bytes());

    buffer.truncate(res_pos);
    buffer
}


