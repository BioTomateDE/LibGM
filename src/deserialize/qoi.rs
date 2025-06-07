use image::{DynamicImage, ImageBuffer, Rgba};
use std::error::Error;

#[derive(Debug)]
struct QoiHeader {
    width: u32,
    height: u32,
    channels: u8,
    colorspace: u8,
}


/// Deserialize a little-endian QOI image into a DynamicImage. Must be RGBA.
pub fn from_bytes_le(bytes: &[u8]) -> Result<DynamicImage, String> {
    // validate minimum size (14 byte header + at least 8 byte end marker)
    if bytes.len() < 22 {
        return Err("Input too small to be a valid QOI image".to_string());
    }

    let header: QoiHeader = parse_qoi_header_le(bytes).map_err(|e| format!("Header error: {}", e))?;
    validate_header(&header)?;

    let pixel_data: &[u8] = &bytes[14..bytes.len() - 8];
    let pixels: ImageBuffer<Rgba<u8>, Vec<u8>> = decode_qoi_pixels(pixel_data, header.width, header.height)
        .map_err(|e| format!("Decoding error: {}", e))?;

    match header.channels {
        3 => Err("RGB images (3 channels) not supported, only RGBA".to_string()),
        4 => Ok(DynamicImage::ImageRgba8(pixels)),
        _ => Err(format!("Unsupported channel count: {}", header.channels)),
    }
}


fn hexdump(bytes: &[u8]) -> String {
    bytes.iter()
        .map(|i| format!("{i:02X}")).collect::<Vec<_>>()
        .join(" ")
}


/// Parse QOI header from little-endian bytes
fn parse_qoi_header_le(bytes: &[u8]) -> Result<QoiHeader, Box<dyn Error>> {
    const MAGIC_HEADER: &[u8; 4] = b"fioq";
    if &bytes[..4] != MAGIC_HEADER {
        return Err(format!("Invalid QOI magic bytes [{}]; expected [{}]", hexdump(&bytes[..4]), hexdump(MAGIC_HEADER)).into());
    }

    let width: u32 = u32::from_le_bytes(bytes[4..8].try_into()?);
    let height: u32 = u32::from_le_bytes(bytes[8..12].try_into()?);
    let channels: u8 = bytes[12];
    let colorspace: u8 = bytes[13];

    Ok(QoiHeader {
        width,
        height,
        channels,
        colorspace,
    })
}


/// Validate header values
fn validate_header(header: &QoiHeader) -> Result<(), String> {
    if header.width == 0 || header.height == 0 {
        return Err("Width and height must be greater than 0".to_string());
    }

    if header.channels != 3 && header.channels != 4 {
        return Err("Only 3 (RGB) or 4 (RGBA) channels are supported".to_string());
    }

    // Check for potential overflow in pixel count
    let total_pixels = header.width as u64 * header.height as u64;
    if total_pixels > (1 << 30) {
        return Err("Image dimensions too large".to_string());
    }

    Ok(())
}

/// Decode QOI pixels with optimized buffer handling
fn decode_qoi_pixels(data: &[u8], width: u32, height: u32) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Box<dyn Error>> {
    let mut pixels: Vec<u8> = vec![0; (width*height*4) as usize];

    // SAFETY: ensure buffer is properly sized before writing
    unsafe {
        decode_qoi_to_rgba(
            data,
            pixels.as_mut_ptr(),
            width as usize,
            height as usize,
        )?;
    }

    let image_buffer = ImageBuffer::from_raw(width, height, pixels)
        .ok_or("Failed to create image buffer from pixel data")?;
    Ok(image_buffer)
}


/// Unsafe but optimized inner decoding loop
unsafe fn decode_qoi_to_rgba(data: &[u8], out_ptr: *mut u8, width: usize, height: usize) -> Result<(), Box<dyn Error>> {
    let mut index: [u8; 256] = [0; 64 * 4];   // QOI index for previously seen pixels
    let mut px = Rgba([0, 0, 0, 255]);
    let mut run: usize = 0;
    let mut out_pos: *mut u8 = out_ptr;
    let mut i: usize = 0;
    
    unsafe {
        while i < data.len() {
            if run > 0 {
                run -= 1;
            } else {
                let b1 = *data.get_unchecked(i);
                i += 1;

                match b1 {
                    // QOI_OP_RGB
                    0b11111110 => {
                        px[0] = *data.get_unchecked(i);
                        px[1] = *data.get_unchecked(i + 1);
                        px[2] = *data.get_unchecked(i + 2);
                        i += 3;
                    }
                    // QOI_OP_RGBA
                    0b11111111 => {
                        px[0] = *data.get_unchecked(i);
                        px[1] = *data.get_unchecked(i + 1);
                        px[2] = *data.get_unchecked(i + 2);
                        px[3] = *data.get_unchecked(i + 3);
                        i += 4;
                    }
                    // QOI_OP_INDEX
                    b1 if (b1 & 0b11000000) == 0b00000000 => {
                        let idx = (b1 & 0b00111111) as usize * 4;
                        px[0] = *index.get_unchecked(idx);
                        px[1] = *index.get_unchecked(idx + 1);
                        px[2] = *index.get_unchecked(idx + 2);
                        px[3] = *index.get_unchecked(idx + 3);
                    }
                    // QOI_OP_DIFF
                    b1 if (b1 & 0b11000000) == 0b01000000 => {
                        px[0] = px[0].wrapping_add(((b1 >> 4) & 0x03).wrapping_sub(2));
                        px[1] = px[1].wrapping_add(((b1 >> 2) & 0x03).wrapping_sub(2));
                        px[2] = px[2].wrapping_add((b1 & 0x03).wrapping_sub(2));
                    }
                    // QOI_OP_LUMA
                    b1 if (b1 & 0b11000000) == 0b10000000 => {
                        let b2 = *data.get_unchecked(i);
                        i += 1;
                        let vg = (b1 & 0b00111111).wrapping_sub(32);
                        px[0] = px[0].wrapping_add(vg.wrapping_add(((b2 >> 4) & 0x0F).wrapping_sub(8)));
                        px[1] = px[1].wrapping_add(vg);
                        px[2] = px[2].wrapping_add(vg.wrapping_add((b2 & 0x0F).wrapping_sub(8)));
                    }
                    // QOI_OP_RUN
                    b1 if (b1 & 0b11000000) == 0b11000000 => {
                        run = (b1 & 0b00111111) as usize;
                    }
                    _ => return Err(format!("Invalid QOI opcode 0x{b1:02X} at position {}", i + 14).into()),
                }

                // ipdate index
                let hash = ((px[0] as usize * 3)
                    + (px[1] as usize * 5)
                    + (px[2] as usize * 7)
                    + (px[3] as usize * 11))
                    % 64;
                *index.get_unchecked_mut(hash * 4) = px[0];
                *index.get_unchecked_mut(hash * 4 + 1) = px[1];
                *index.get_unchecked_mut(hash * 4 + 2) = px[2];
                *index.get_unchecked_mut(hash * 4 + 3) = px[3];
            }

            // write pixel
            *out_pos = px[0];
            *out_pos.add(1) = px[1];
            *out_pos.add(2) = px[2];
            *out_pos.add(3) = px[3];
            out_pos = out_pos.add(4);
        }
    }

    // Validate we decoded exactly width*height pixels
    let decoded_pixels: usize = (out_pos as usize - out_ptr as usize) / 4;
    if decoded_pixels != width * height {
        return Err(format!(
            "Pixel count mismatch: expected {}, got {}",
            width * height, decoded_pixels,
        ).into());
    }

    Ok(())
}

