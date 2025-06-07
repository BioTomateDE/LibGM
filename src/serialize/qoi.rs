use image::DynamicImage;
use std::io::{Cursor, Write};

/// Serialize a DynamicImage to little-endian QOI bytes
pub fn to_bytes_le(image: &DynamicImage) -> Result<Vec<u8>, String> {
    let (width, height) = (image.width(), image.height());
    if width == 0 || height == 0 {
        return Err("Image dimensions must be greater than 0".to_string());
    }

    // get the raw pixels without conversion if already in RGBA8
    let rgba_image = match image {
        DynamicImage::ImageRgba8(img) => img,
        _ => &image.to_rgba8(),   // only convert if necessary
    };

    // preallocate buffer with estimated size (header + pixels + end marker)
    let capacity: usize = 14 + (width as usize * height as usize * 5) + 8;   // worst case
    let mut buffer: Vec<u8> = Vec::with_capacity(capacity);
    let mut cursor = Cursor::new(&mut buffer);

    // write header (little-endian)
    write_qoi_header_le(&mut cursor, width, height, 4, 0)
        .map_err(|e| format!("Header write failed: {}", e))?;

    // encode pixels
    encode_qoi_pixels(&rgba_image, &mut cursor)
        .map_err(|e| format!("Pixel encoding failed: {}", e))?;

    // write end marker
    cursor.write_all(&[0, 0, 0, 0, 0, 0, 0, 1])
        .map_err(|e| format!("Failed to write end marker: {}", e))?;

    Ok(buffer)
}


/// Write QOI header in little-endian format
fn write_qoi_header_le(w: &mut Cursor<&mut Vec<u8>>, width: u32, height: u32, channels: u8, colorspace: u8) -> Result<(), std::io::Error> {
    w.write_all(b"fioq")?;    // reversed because little endian
    w.write_all(&width.to_le_bytes())?;
    w.write_all(&height.to_le_bytes())?;
    w.write_all(&[channels, colorspace])?;
    Ok(())
}


/// Optimized QOI pixel encoding
fn encode_qoi_pixels(image: &image::RgbaImage, w: &mut Cursor<&mut Vec<u8>>) -> Result<(), std::io::Error> {
    let mut index = [[0u8; 4]; 64];   // QOI color index
    let mut prev_px: [u8; 4] = [0, 0, 0, 255];
    let mut run: u8 = 0;

    // SAFETY: We use unsafe for the pixel access hot loop
    for pixel in image.pixels() {
        let px: [u8; 4] = pixel.0;
        
        if px == prev_px && run < 61 {
            run += 1;
            continue;
        }

        // Flush run if needed
        if run > 0 {
            w.write_all(&[0b11000000 | (run - 1)])?;
            run = 0;
        }

        let hash = (px[0] as usize * 3
            + px[1] as usize * 5
            + px[2] as usize * 7
            + px[3] as usize * 11)
            % 64;

        if index[hash] == px {
            // QOI_OP_INDEX
            w.write_all(&[hash as u8])?;
        } else {
            index[hash] = px;

            let vr = px[0].wrapping_sub(prev_px[0]);
            let vg = px[1].wrapping_sub(prev_px[1]);
            let vb = px[2].wrapping_sub(prev_px[2]);
            let va = px[3].wrapping_sub(prev_px[3]);

            if va == 0 {
                // Handle RGB channels
                if vr >= 0xFE || vg >= 0xFE || vb >= 0xFE {
                    // QOI_OP_RGB
                    w.write_all(&[0b11111110, px[0], px[1], px[2]])?;
                } else {
                    let dr = vr.wrapping_add(2);
                    let dg = vg.wrapping_add(2);
                    let db = vb.wrapping_add(2);

                    if dr < 4 && dg < 4 && db < 4 {
                        // QOI_OP_DIFF
                        w.write_all(&[0b01000000 | (dr << 4) | (dg << 2) | db])?;
                    } else {
                        let dg = vg.wrapping_add(32);
                        let dr_dg = vr.wrapping_sub(vg).wrapping_add(8);
                        let db_dg = vb.wrapping_sub(vg).wrapping_add(8);

                        if dg < 64 && dr_dg < 16 && db_dg < 16 {
                            // QOI_OP_LUMA
                            w.write_all(&[
                                0b10000000 | dg,
                                (dr_dg << 4) | db_dg,
                            ])?;
                        } else {
                            // QOI_OP_RGB
                            w.write_all(&[0b11111110, px[0], px[1], px[2]])?;
                        }
                    }
                }
            } else {
                // QOI_OP_RGBA
                w.write_all(&[0b11111111, px[0], px[1], px[2], px[3]])?;
            }
        }

        prev_px = px;
    }

    // Flush any remaining run
    if run > 0 {
        w.write_all(&[0b11000000 | (run - 1)])?;
    }

    Ok(())
}

