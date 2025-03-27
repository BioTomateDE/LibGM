use std::cmp::max;
use crate::deserialize::chunk_reading::UTChunk;
use crate::deserialize::general_info::UTGeneralInfo;
use crate::printing::hexdump;

pub struct UTEmbeddedTexture {
    scaled: u32,
    generated_mips: Option<u32>,
    texture_block_size: Option<u32>,
    texture_width: Option<i32>,
    texture_height: Option<i32>,
    index_in_group: Option<i32>,
    texture_data: Image,
}


pub enum Image {
    Png(Vec<u8>),

}


pub fn parse_chunk_TXTR(mut chunk: UTChunk, general_info: &UTGeneralInfo) -> Result<Vec<UTEmbeddedTexture>, String> {
    let texture_count: usize = chunk.read_usize()?;
    let mut texture_pointers: Vec<usize> = Vec::with_capacity(texture_count);

    for _ in 0..texture_count {
        texture_pointers.push(chunk.read_usize()? - chunk.abs_pos);
    }

    let mut textures: Vec<UTEmbeddedTexture> = Vec::with_capacity(texture_count);
    for _ in 0..texture_count {
        let texture: UTEmbeddedTexture = parse_texture(&mut chunk, general_info)?;
        textures.push(texture);
    }

    Ok(textures)
}


fn parse_texture(chunk: &mut UTChunk, general_info: &UTGeneralInfo) -> Result<UTEmbeddedTexture, String> {
    let scaled: u32 = chunk.read_u32()?;
    let mut generated_mips: Option<u32> = None;
    let mut texture_block_size: Option<u32> = None;
    let mut texture_width: Option<i32> = None;
    let mut texture_height: Option<i32> = None;
    let mut index_in_group: Option<i32> = None;
    // reader directory {}

    if general_info.is_version_at_least(2, 0, 6, 0) {
        generated_mips = Some(chunk.read_u32()?);
    }
    if general_info.is_version_at_least(2022, 3, 0, 0) {
        texture_block_size = Some(chunk.read_u32()?);
    }
    if general_info.is_version_at_least(2022, 9, 0, 0) {
        texture_width = Some(chunk.read_i32()?);
        texture_height = Some(chunk.read_i32()?);
        index_in_group = Some(chunk.read_i32()?);
    }

    let texture_data: Image = read_raw_texture(chunk, general_info)?;

    Ok(UTEmbeddedTexture {
        scaled,
        generated_mips,
        texture_block_size,
        texture_width,
        texture_height,
        index_in_group,
        texture_data,
    })
}



fn read_raw_texture(chunk: &mut UTChunk, general_info: &UTGeneralInfo) -> Result<Image, String> {
    static MAGIC_PNG_HEADER: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
    static MAGIC_BZ2_QOI_HEADER: &[u8] = "2zoq".as_bytes();
    static MAGIC_QOI_HEADER: &[u8] = "fioq".as_bytes();

    let start_position: usize = chunk.file_index;
    let header: [u8; 8] = match chunk.data.get(chunk.file_index .. chunk.file_index+8) {
        Some(bytes) => bytes.try_into().unwrap(),
        None => return Err(format!(
            "Unexpected end of chunk while trying to read headers of texture at position {} in chunk 'TXTR'.",
            start_position
        )),
    };

    if header == MAGIC_PNG_HEADER {
        // Parse PNG
        loop {
            let len: usize = chunk.read_usize_big_endian()?;
            let type_: usize = chunk.read_usize_big_endian()?;
            chunk.file_index += len + 4;
            if type_ == 0x49454E44 {
                break;
            }
        }


        let bytes: [u8] = chunk.data[start_position .. chunk.file_index];
        // png image size checks etc. {}
        Ok(Image::Png(bytes.to_vec()))
    }
    else if header.starts_with(MAGIC_BZ2_QOI_HEADER) {
        // Parse QOI + BZip2
        chunk.file_index += 8;      // skip past (start of) header
        let mut serialized_uncompressed_length: i32 = -1;
        let mut header_size: usize = 8;
        if general_info.is_version_at_least(2022, 5, 0, 0) {
            serialized_uncompressed_length = chunk.read_i32()?;
            header_size = 12;
        }

        let end_of_bz2_stream: usize = find_end_of_bz2_stream(chunk)?;
        let compressed_length: usize = end_of_bz2_stream - start_position - header_size;    // maybe negative?? shouldn't ever be though i think
        let width: u32 = header[4] as u32 | (header[5] << 8) as u32;
        let height: u32 = header[6] as u32 | (header[7] << 8) as u32;

        Ok(Image {data: vec![vec![(6, 9, 6, 9)]]})
    }
    else if header.starts_with(MAGIC_QOI_HEADER) {
        // Parse QOI
        Ok(Image {data: vec![vec![(6, 9, 6, 9)]]})
    }
    else {
        let dump: String = hexdump(&header, 0, None)?;
        Err(format!("Invalid image header [{dump}] while parsing texture at position {start_position} in chunk 'TXTR'."))
    }
}


fn find_end_of_bz2_stream(ut_chunk: &mut UTChunk) -> Result<usize, String> {
    let stream_start_position: usize = ut_chunk.file_index;
    // Read backwards from the max end of stream position, in up to 256-byte chunks.
    // We want to find the end of nonzero data.
    static MAX_CHUNK_SIZE: usize = 256;

    let mut chunk_start_position: usize = max(stream_start_position, ut_chunk.data_len - MAX_CHUNK_SIZE);
    let chunk_size: usize = ut_chunk.data_len - chunk_start_position;
    loop {
        ut_chunk.file_index = chunk_start_position;
        let chunk_data: [u8] = ut_chunk.data[ut_chunk.file_index .. ut_chunk.file_index + chunk_size];
        ut_chunk.file_index += chunk_size;

        // find first nonzero byte at end of stream
        let mut position: isize = chunk_size as isize - 1;
        while position >= 0 && chunk_data[position] == 0 {
            position -= 1;
        }

        // If we're at nonzero data, then invoke search for footer magic
        if position >= 0 && chunk_data[position] != 0 {
            let end_data_position: isize = chunk_start_position as isize + position + 1;
            return Ok(find_end_of_bz2_search(ut_chunk, end_data_position as usize)?)
        }

        // move backwards to next chunk
        chunk_start_position = max(stream_start_position, chunk_start_position - MAX_CHUNK_SIZE);
        if chunk_start_position <= stream_start_position {
            return Err("Failed to find nonzero data while trying to find end of bz2 stream.".to_string())
        }
    }
}


/// function written by chatgpt; unverified
fn find_end_of_bz2_search(ut_chunk: &mut UTChunk, end_data_position: usize) -> Result<usize, String> {
    static MAGIC_BZ2_FOOTER: [u8; 6] = [0x17, 0x72, 0x45, 0x38, 0x50, 0x90];

    // Ensure we don't read past the data bounds
    let start_position = end_data_position.saturating_sub(16);
    if start_position >= ut_chunk.data_len {
        return Err("Start position out of bounds".to_string());
    }

    // Extract the last 16 bytes (or fewer if near the start of data)
    let data = &ut_chunk.data[start_position..end_data_position];

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

