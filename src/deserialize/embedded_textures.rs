use std::cmp::max;
use std::io::Read;
use crate::deserialize::chunk_reading::GMChunk;
use crate::deserialize::general_info::GMGeneralInfo;
use crate::printing::hexdump;
use image;
use bzip2::read::BzDecoder;
use image::{DynamicImage, ImageBuffer, Rgba};

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

pub const MAGIC_PNG_HEADER: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
pub const MAGIC_BZ2_QOI_HEADER: &[u8] = "2zoq".as_bytes();
pub const MAGIC_QOI_HEADER: &[u8] = "fioq".as_bytes();


pub fn parse_chunk_txtr(chunk: &mut GMChunk, general_info: &GMGeneralInfo) -> Result<Vec<GMEmbeddedTexture>, String> {
    chunk.cur_pos = 0;
    let texture_count: usize = chunk.read_usize()?;
    let mut texture_pointers: Vec<usize> = Vec::with_capacity(texture_count);

    for _ in 0..texture_count {
        texture_pointers.push(chunk.read_usize()? - chunk.abs_pos);
    }

    let mut textures: Vec<GMEmbeddedTexture> = Vec::with_capacity(texture_count);
    for texture_start_position in texture_pointers {
        chunk.cur_pos = texture_start_position;
        let texture: GMEmbeddedTexture = parse_texture(chunk, general_info)?;
        textures.push(texture);
    }

    Ok(textures)
}


fn parse_texture(chunk: &mut GMChunk, general_info: &GMGeneralInfo) -> Result<GMEmbeddedTexture, String> {
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
    let image: Option<DynamicImage> = if texture_abs_start_position == 0 { None } else {
        let texture_start_position: usize = texture_abs_start_position.checked_sub(chunk.abs_pos).ok_or_else(|| format!(
            "Trying to subtract with overflow for absolute texture start position {0} (0x{0:08X}) with chunk position {1}",
            texture_abs_start_position, chunk.abs_pos,
        ))?;
        chunk.cur_pos = texture_start_position;
        Some(read_raw_texture(chunk, general_info)?)
    };
    
    Ok(GMEmbeddedTexture {
        scaled,
        generated_mips,
        texture_width,
        texture_height,
        index_in_group,
        image,
    })
}


fn read_raw_texture(chunk: &mut GMChunk, general_info: &GMGeneralInfo) -> Result<DynamicImage, String> {
    let start_position: usize = chunk.cur_pos;
    let header: [u8; 8] = match chunk.data.get(chunk.cur_pos.. chunk.cur_pos+8) {
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
        let image: DynamicImage = image::load_from_memory(&bytes)
            .map_err(|e| format!("Could not parse PNG image for texture page at position {start_position} in chunk 'TXTR': \"{e}\""
        ))?;
        Ok(image)
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
        let compressed_length: usize = end_of_bz2_stream - start_position - header_size;    // maybe negative?? shouldn't ever be though i think
        let width: usize = ((header[4] as u32) | ((header[5] as u32) << 8)) as usize;
        let height: usize = ((header[6] as u32) | ((header[7] as u32) << 8)) as usize;

        // read entire image (excluding bz2 header) to byte array
        chunk.cur_pos = start_position + header_size;
        let raw_image_data: &[u8] = &chunk.data[chunk.cur_pos.. chunk.cur_pos + compressed_length];
        chunk.cur_pos += compressed_length;
        let image: DynamicImage = image_from_bz2_qoi(&raw_image_data, width, height)
            .map_err(|e| format!("{e} while parsing QOI image with dimensions {width}x{height} at position {start_position} in chunk 'TXTR'"))?;
        Ok(image)
    }
    else if header.starts_with(MAGIC_QOI_HEADER) {
        // Parse QOI
        return Err(format!("Unhandled QOI image at position {} in chunk 'TXTR'", chunk.cur_pos));
        // image_from_qoi(chunk.data[chunk..])
    }
    else {
        let dump: String = hexdump(&header, 0, None)?;
        Err(format!("Invalid image header [{dump}] while parsing texture at position {start_position} in chunk 'TXTR'"))
    }
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


fn image_from_bz2_qoi(raw_image_data: &[u8], width: usize, height: usize) -> Result<DynamicImage, String> {
    let mut decoder: BzDecoder<&[u8]> = BzDecoder::new(raw_image_data);
    let mut decompressed_data: Vec<u8> = Vec::new();
    decoder.read_to_end(&mut decompressed_data)
        .map_err(|e| format!("Could not decode BZip2 data: \"{e}\""))?;
    image_from_qoi(&decompressed_data, width, height)
}

fn image_from_qoi(raw_image_data: &Vec<u8>, width: usize, height: usize) -> Result<DynamicImage, String> {
    let (_header, bytes) = qoi::decode_to_vec(raw_image_data)
        .map_err(|e| format!("Could not decode QOI image: {e}"))?;
    
    let image = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(width as u32, height as u32, bytes)
        .ok_or("Could not construct image::RgbaImage from pixel data")?;
    
    Ok(DynamicImage::ImageRgba8(image))
}

