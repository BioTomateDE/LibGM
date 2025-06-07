use std::cmp::max;
use std::io::Read;
use std::sync::Mutex;
use crate::deserialize::chunk_reading::GMChunk;
use crate::deserialize::general_info::GMGeneralInfo;
use crate::printing::hexdump;
use image;
use bzip2::read::BzDecoder;
use image::{DynamicImage, RgbaImage};

#[derive(Debug, Clone, PartialEq)]
pub struct GMEmbeddedTexture {
    /// not sure what `scaled` actually is
    pub scaled: u32,
    /// same with this
    pub generated_mips: Option<u32>,
    pub texture_width: Option<i32>,
    pub texture_height: Option<i32>,
    pub index_in_group: Option<i32>,
    pub image: Option<DynamicImage>,
}

struct GMEmbeddedTextureRaw<'a> {
    scaled: u32,
    generated_mips: Option<u32>,
    texture_width: Option<i32>,
    texture_height: Option<i32>,
    index_in_group: Option<i32>,
    image: Option<RawImage<'a>>,
}

pub const MAGIC_PNG_HEADER: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
pub const MAGIC_BZ2_QOI_HEADER: &[u8] = "2zoq".as_bytes();
pub const MAGIC_QOI_HEADER: &[u8] = "fioq".as_bytes();


struct RawImage<'a> {
    data: &'a [u8],
    position_in_data: usize,
    kind: RawImageKind,
}

enum RawImageKind {
    Png,
    Bz2Qoi,
    // Qoi,
}


pub fn parse_chunk_txtr(chunk: &mut GMChunk, general_info: &GMGeneralInfo) -> Result<Vec<GMEmbeddedTexture>, String> {
    chunk.cur_pos = 0;
    let texture_count: usize = chunk.read_usize()?;
    let mut texture_pointers: Vec<usize> = Vec::with_capacity(texture_count);

    for _ in 0..texture_count {
        texture_pointers.push(chunk.read_usize()? - chunk.abs_pos);
    }

    let mut textures_raw: Vec<GMEmbeddedTextureRaw> = Vec::with_capacity(texture_count);
    for texture_start_position in texture_pointers {
        chunk.cur_pos = texture_start_position;

        let scaled: u32 = chunk.read_u32()?;
        let mut generated_mips: Option<u32> = None;
        let mut texture_width: Option<i32> = None;
        let mut texture_height: Option<i32> = None;
        let mut index_in_group: Option<i32> = None;
        // reader directory {}

        if general_info.is_version_at_least(2, 0, 6, 0) {
            generated_mips = Some(chunk.read_u32()?);
        }
        // if general_info.is_version_at_least(2022, 3, 0, 0) {
        //     texture_block_size = Some(chunk.read_u32()?);
        // }
        if general_info.is_version_at_least(2022, 9, 0, 0) {
            texture_width = Some(chunk.read_i32()?);
            texture_height = Some(chunk.read_i32()?);
            index_in_group = Some(chunk.read_i32()?);
        }

        let texture_abs_start_position: usize = chunk.read_usize()?;
        // can be zero if the texture is "external"
        let image: Option<RawImage> = if texture_abs_start_position == 0 { None } else {
            let texture_start_position: usize = texture_abs_start_position.checked_sub(chunk.abs_pos).ok_or_else(|| format!(
                "Trying to subtract with overflow for absolute texture start position {0} (0x{0:08X}) with chunk position {1}",
                texture_abs_start_position, chunk.abs_pos,
            ))?;
            chunk.cur_pos = texture_start_position;
            Some(read_raw_texture(chunk, general_info)?)
        };

        textures_raw.push(GMEmbeddedTextureRaw {
            scaled,
            generated_mips,
            texture_width,
            texture_height,
            index_in_group,
            image,
        });
    }

    let textures: Mutex<Vec<GMEmbeddedTexture>> = Mutex::new(Vec::with_capacity(texture_count));
    textures_raw.iter().try_for_each(|raw_texture| {
        let image: Option<DynamicImage> = if let Some(ref raw_img) = raw_texture.image {
            Some(read_raw_image(raw_img)?)
        } else {
            None
        };
        textures.lock().unwrap().push(GMEmbeddedTexture {
            scaled: raw_texture.scaled,
            generated_mips: raw_texture.generated_mips,
            texture_width: raw_texture.texture_width,
            texture_height: raw_texture.texture_height,
            index_in_group: raw_texture.index_in_group,
            image,
        });
        Ok(())
    }).map_err(|e: String| format!("Error while parsing texture page images: {e}"))?;

    let textures = textures.into_inner()
        .map_err(|e| format!("Could not acquire textures Mutex: {e}"))?;

    // for (i, texture) in textures.iter().enumerate() {
    //     if let Some(ref img) = texture.image {
    //         use std::path::PathBuf;
    //         let path = PathBuf::from(format!("./_texture_pages/{i}.png"));
    //         img.save(path).map_err(|e| format!("Could not save image #{i}: {e}"))?;
    //     }
    // }

    Ok(textures)
}


fn read_raw_texture<'a>(chunk: &mut GMChunk<'a>, general_info: &GMGeneralInfo) -> Result<RawImage<'a>, String> {
    let start_position: usize = chunk.cur_pos;
    let header: [u8; 8] = match chunk.data.get(chunk.cur_pos..chunk.cur_pos+8) {
        Some(bytes) => bytes.try_into().unwrap(),
        None => return Err(format!(
            "Unexpected end of chunk while trying to read headers of texture at position {} in chunk 'TXTR'",
            start_position,
        )),
    };

    if header == MAGIC_PNG_HEADER {
        // Parse PNG
        chunk.cur_pos += 8;  // skip header
        loop {
            let len: usize = chunk.read_usize_big_endian(true)?;
            let type_: usize = chunk.read_usize_big_endian(false)?;
            chunk.cur_pos += len + 4;
            if type_ == 0x49454E44 {    // no idea lol
                break;
            }
        }
        
        let bytes: &[u8] = &chunk.data.get(start_position .. chunk.cur_pos).ok_or_else(|| format!(
            "Trying to read PNG data out of bounds in chunk 'TXTR' at position {}: {} > {}",
            start_position, chunk.cur_pos, chunk.data.len(),
        ))?;
        // png image size checks {~~}
        Ok(RawImage {
            data: bytes,
            position_in_data: start_position,
            kind: RawImageKind::Png,
        })
    }
    else if header.starts_with(MAGIC_BZ2_QOI_HEADER) {
        // Parse QOI + BZip2
        chunk.cur_pos += 8;      // skip past (start of) header
        let mut header_size: usize = 8;
        if general_info.is_version_at_least(2022, 5, 0, 0) {
            let _serialized_uncompressed_length = chunk.read_usize()?;    // maybe handle negative numbers?
            header_size = 12;
        }

        let end_of_bz2_stream: usize = find_end_of_bz2_stream(chunk)?;
        // read entire image (excluding bz2 header) to byte array
        chunk.cur_pos = start_position + header_size;
        let raw_image_data: &[u8] = &chunk.data[start_position+header_size .. end_of_bz2_stream];
        chunk.cur_pos = end_of_bz2_stream;
        Ok(RawImage {
            data: raw_image_data,
            position_in_data: start_position,
            kind: RawImageKind::Bz2Qoi,
        })
    }
    else if header.starts_with(MAGIC_QOI_HEADER) {
        // Parse QOI
        return Err(format!("Raw QOI images not yet implemented at position {} in chunk 'TXTR'", chunk.cur_pos));
        // image_from_qoi(chunk.data[chunk..])
    }
    else {
        let dump: String = hexdump(&header, 0, None)?;
        Err(format!("Invalid image header [{dump}] while parsing texture at position {start_position} in chunk 'TXTR'"))
    }
}


fn read_raw_image(raw_image: &RawImage) -> Result<DynamicImage, String> {
    let dynamic_image: DynamicImage = match &raw_image.kind {
        RawImageKind::Png => {
            image::load_from_memory(&raw_image.data)
                .map_err(|e| format!("Could not parse PNG image for texture page at position {} in chunk 'TXTR': {e}", raw_image.position_in_data))?
        }
        RawImageKind::Bz2Qoi => {
            image_from_bz2_qoi(&raw_image.data)
                .map_err(|e| format!("Could not parse Bz2 QOI image for texture page at position {} in chunk 'TXTR': {e}", raw_image.position_in_data))?
        }
    };
    Ok(dynamic_image)
}


fn find_end_of_bz2_stream(gm_chunk: &mut GMChunk) -> Result<usize, String> {
    let stream_start_position: usize = gm_chunk.cur_pos;
    // Read backwards from the max end of stream position, in up to 256-byte chunks.
    // We want to find the end of nonzero data.
    static MAX_CHUNK_SIZE: usize = 256;

    let mut chunk_start_position: usize = max(stream_start_position, gm_chunk.data.len() - MAX_CHUNK_SIZE);
    let chunk_size: usize = gm_chunk.data.len() - chunk_start_position;
    loop {
        gm_chunk.cur_pos = chunk_start_position;
        let chunk_data: &[u8] = &gm_chunk.data[gm_chunk.cur_pos.. gm_chunk.cur_pos + chunk_size];
        gm_chunk.cur_pos += chunk_size;

        // find first nonzero byte at end of stream
        let mut position: isize = chunk_size as isize - 1;
        while position >= 0 && chunk_data[position as usize] == 0 {
            position -= 1;
        }

        // If we're at nonzero data, then invoke search for footer magic
        if position >= 0 && chunk_data[position as usize] != 0 {
            let end_data_position: isize = chunk_start_position as isize + position + 1;
            return Ok(find_end_of_bz2_search(gm_chunk, end_data_position as usize)?)
        }

        // move backwards to next chunk
        chunk_start_position = max(stream_start_position, chunk_start_position - MAX_CHUNK_SIZE);
        if chunk_start_position <= stream_start_position {
            return Err("Failed to find nonzero data while trying to find end of bz2 stream".to_string())
        }
    }
}


/// function written by chatgpt; unverified
fn find_end_of_bz2_search(gm_chunk: &mut GMChunk, end_data_position: usize) -> Result<usize, String> {
    static MAGIC_BZ2_FOOTER: [u8; 6] = [0x17, 0x72, 0x45, 0x38, 0x50, 0x90];

    // Ensure we don't read past the data bounds
    let start_position = end_data_position.saturating_sub(16);
    if start_position >= gm_chunk.data.len() {
        return Err("Start position out of bounds".to_string());
    }

    // Extract the last 16 bytes (or fewer if near the start of data)
    let data = &gm_chunk.data[start_position..end_data_position];

    // BZ2 footer magic bytes
    let footer_magic: &[u8] = &MAGIC_BZ2_FOOTER;
    let mut search_start_position = data.len() as isize - 1;
    let mut search_start_bit_position = 0;

    while search_start_position >= 0 {
        let mut found_match = false;
        let mut bit_position = search_start_bit_position;
        let mut search_position = search_start_position;
        let mut magic_bit_position = 0;
        let mut magic_position = footer_magic.len() as isize - 1;

        while search_position >= 0 {
            let current_byte = data[search_position as usize];
            let magic_byte = footer_magic[magic_position as usize];

            // Extract specific bits from the current and magic bytes
            let current_bit = (current_byte & (1 << bit_position)) != 0;
            let magic_current_bit = (magic_byte & (1 << magic_bit_position)) != 0;

            if current_bit != magic_current_bit {
                break;
            }

            // Progress through magic bits
            magic_bit_position += 1;
            if magic_bit_position >= 8 {
                magic_bit_position = 0;
                magic_position -= 1;
            }

            // If we matched the entire magic footer, we found the end
            if magic_position < 0 {
                found_match = true;
                break;
            }

            // Move to the next bit in the search data
            bit_position += 1;
            if bit_position >= 8 {
                bit_position = 0;
                search_position -= 1;
            }
        }

        if found_match {
            const FOOTER_BYTE_LENGTH: usize = 10;
            let mut end_of_bz2_stream_position = (search_position + FOOTER_BYTE_LENGTH as isize) as usize;

            // If the footer started mid-byte, account for unused padding bits
            if bit_position != 7 {
                end_of_bz2_stream_position += 1;
            }

            return Ok(start_position + end_of_bz2_stream_position);
        }

        // Move to the next bit for searching
        search_start_bit_position += 1;
        if search_start_bit_position >= 8 {
            search_start_bit_position = 0;
            search_start_position -= 1;
        }
    }

    Err("Failed to find BZip2 footer magic".to_string())
}


fn image_from_bz2_qoi(raw_image_data: &[u8]) -> Result<DynamicImage, String> {
    let mut decoder: BzDecoder<&[u8]> = BzDecoder::new(raw_image_data);
    let mut decompressed_data: Vec<u8> = Vec::new();
    decoder.read_to_end(&mut decompressed_data)
        .map_err(|e| format!("Could not decode BZip2 data: \"{e}\""))?;

    let (header, rgba_data) = qoi::decode_to_vec(decompressed_data)
        .map_err(|e| format!("Could not decode QOI image: {e}"))?;

    let image = RgbaImage::from_vec(u32::from(header.width), u32::from(header.height), rgba_data)
        .ok_or("Could not convert raw RGBA bytes into ImageBuffer")?;
    Ok(DynamicImage::ImageRgba8(image))
}
