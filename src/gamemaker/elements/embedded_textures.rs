use crate::gamemaker::data::Endianness;
use crate::gamemaker::deserialize::DataReader;
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::qoi;
use crate::gamemaker::serialize::DataBuilder;
use crate::gamemaker::serialize::traits::GMSerializeIfVersion;
use crate::prelude::*;
use crate::util::fmt::hexdump;
use bzip2::read::BzDecoder;
use image;
use image::{DynamicImage, ImageFormat};
use std::borrow::Cow;
use std::cmp::max;
use std::io::{Cursor, Read};

pub const MAGIC_PNG_HEADER: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
pub const MAGIC_BZ2_QOI_HEADER: &[u8; 4] = b"2zoq";
pub const MAGIC_QOI_HEADER: &[u8; 4] = b"fioq";

#[derive(Debug, Clone)]
pub struct GMEmbeddedTextures {
    pub texture_pages: Vec<GMEmbeddedTexture>,
    pub exists: bool,
}

impl GMChunkElement for GMEmbeddedTextures {
    fn stub() -> Self {
        Self { texture_pages: vec![], exists: false }
    }
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMEmbeddedTextures {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let mut texture_pages: Vec<GMEmbeddedTexture> = reader.read_pointer_list()?;
        for i in 0..texture_pages.len() {
            // Find next element start position
            let mut max_end_of_stream_pos = reader.chunk.end_pos;
            for texture_page in &texture_pages[i + 1..] {
                let Some(ref img) = texture_page.image else {
                    continue;
                };
                let GMImage::NotYetDeserialized(blob_pos) = img else {
                    bail!("GMImage enum variant is somehow not `NotYetDeserialized`");
                };
                max_end_of_stream_pos = *blob_pos as usize;
                break;
            }

            let texture_page: &mut GMEmbeddedTexture = &mut texture_pages[i];
            let Some(ref mut gm_image) = texture_page.image else {
                continue; // Texture is external
            };
            let GMImage::NotYetDeserialized(blob_position) = gm_image else {
                bail!("GMImage enum variant is somehow not `NotYetDeserialized`");
            };
            reader.cur_pos = *blob_position as usize;
            *gm_image = read_raw_texture(reader, max_end_of_stream_pos, texture_page.texture_block_size)?;
        }

        reader.align(4)?;
        Ok(Self { texture_pages, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        if !self.exists {
            return Ok(());
        }
        builder.write_usize(self.texture_pages.len())?;
        let pointer_list_start_pos: usize = builder.len();
        for _ in 0..self.texture_pages.len() {
            builder.write_u32(0xDEADC0DE);
        }

        let mut texture_block_size_placeholders: Vec<usize> = vec![0; self.texture_pages.len()];

        for (i, texture_page) in self.texture_pages.iter().enumerate() {
            builder.overwrite_usize(builder.len(), pointer_list_start_pos + i * 4)?;
            builder.write_u32(texture_page.scaled);
            texture_page
                .generated_mips
                .serialize_if_gm_ver(builder, "Generated Mipmap levels", (2, 0, 6))?;
            if builder.is_gm_version_at_least((2022, 3)) {
                texture_block_size_placeholders[i] = builder.len();
                // Placeholder for texture block size. will not be overwritten if external
                builder.write_u32(
                    texture_page
                        .texture_block_size
                        .context("Texture block size not set in 2022.3+")?,
                );
            }
            texture_page
                .data_2022_9
                .serialize_if_gm_ver(builder, "Texture Page 2022.9 data", (2022, 9))?;

            if texture_page.image.is_some() {
                builder.write_pointer(&texture_page.image)?;
            } else {
                builder.write_u32(0); // External texture
            }
        }

        for (i, texture_page) in self.texture_pages.iter().enumerate() {
            if let Some(ref img) = texture_page.image {
                builder.align(0x80);
                builder.resolve_pointer(&texture_page.image)?;
                let start_pos: usize = builder.len();
                img.serialize(builder)?;
                if builder.is_gm_version_at_least((2022, 3)) {
                    let length: usize = builder.len() - start_pos;
                    builder.overwrite_usize(length, texture_block_size_placeholders[i])?
                }
            }
        }

        builder.align(4);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
#[repr(C)] // Need explicit layout so memory addresses for gm pointers don't collide
pub struct GMEmbeddedTexture {
    /// not sure what `scaled` actually is
    pub scaled: u32,
    /// The amount of generated mipmap levels. Present in 2.0.6+
    pub generated_mips: Option<u32>,
    /// Size of the texture attached to this texture page in bytes. Present in 2022.3+
    pub texture_block_size: Option<u32>,
    pub data_2022_9: Option<GMEmbeddedTexture2022_9>,
    pub image: Option<GMImage>,
}
impl GMElement for GMEmbeddedTexture {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let scaled = reader.read_u32()?;
        let generated_mips: Option<u32> = reader.deserialize_if_gm_version((2, 0, 6))?;
        let texture_block_size: Option<u32> = reader.deserialize_if_gm_version((2022, 3))?;
        let data_2022_9: Option<GMEmbeddedTexture2022_9> = reader.deserialize_if_gm_version((2022, 9))?;

        let texture_data_start_pos = reader.read_u32()?;
        let image: Option<GMImage> = if texture_data_start_pos == 0 {
            None // Texture_data_start_pos is zero if the texture is "external"
        } else {
            Some(GMImage::NotYetDeserialized(texture_data_start_pos))
        };

        Ok(GMEmbeddedTexture {
            scaled,
            generated_mips,
            texture_block_size,
            data_2022_9,
            image,
        })
    }

    fn serialize(&self, _builder: &mut DataBuilder) -> Result<()> {
        unreachable!(
            "[internal error] GMEmbeddedTexture::serialize is not supported; use GMEmbeddedTextures::serialize instead"
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMEmbeddedTexture2022_9 {
    pub texture_width: i32,
    pub texture_height: i32,
    pub index_in_group: i32,
}
impl GMElement for GMEmbeddedTexture2022_9 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let texture_width = reader.read_i32()?;
        let texture_height = reader.read_i32()?;
        let index_in_group = reader.read_i32()?;
        Ok(Self { texture_width, texture_height, index_in_group })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(self.texture_width);
        builder.write_i32(self.texture_height);
        builder.write_i32(self.index_in_group);
        Ok(())
    }
}

fn read_raw_texture(
    reader: &mut DataReader,
    max_end_of_stream_pos: usize,
    texture_block_size: Option<u32>,
) -> Result<GMImage> {
    reader.align(0x80)?;
    let start_position: usize = reader.cur_pos;
    let header: [u8; 8] = reader.read_bytes_const().cloned().map_err(|e| {
        format!("Trying to read headers {e} at position {start_position} while parsing images of texture pages")
    })?;

    if header == MAGIC_PNG_HEADER {
        // Parse PNG
        loop {
            let len: u32 = read_u32_be(reader, "Trying to read PNG chunk length")?;
            let r#type: u32 = read_u32_be(reader, "Trying to read PNG chunk type")?;
            reader.cur_pos += len as usize + 4;
            if r#type == 0x49454E44 {
                // "IEND"
                break;
            }
        }

        let data_length: usize = reader.cur_pos - start_position;
        if let Some(expected_size) = texture_block_size {
            if expected_size as usize != data_length {
                bail!(
                    "Texture Page Entry specified texture block size {}; actually detected length {} for PNG Image data",
                    expected_size,
                    data_length,
                );
            }
        }

        reader.cur_pos = start_position;
        let bytes: &[u8] = reader
            .read_bytes_dyn(data_length)
            .map_err(|e| format!("Trying to read PNG image data {e}"))?;
        // Png image size checks {~~}
        let image = GMImage::from_png(bytes.to_vec());
        Ok(image)
    } else if header.starts_with(MAGIC_BZ2_QOI_HEADER) {
        // Parse QOI + BZip2
        let mut header_size: usize = 8;
        let mut uncompressed_size = None;
        if reader.general_info.is_version_at_least((2022, 5)) {
            uncompressed_size = Some(reader.read_u32()?);
            header_size = 12;
        }

        let bz2_stream_end: usize = find_end_of_bz2_stream(reader, max_end_of_stream_pos)?;
        let bz2_stream_length: usize = bz2_stream_end - start_position - header_size;
        if let Some(expected_size) = texture_block_size {
            if expected_size as usize != bz2_stream_length + header_size {
                bail!(
                    "Texture Page Entry specified texture block size {}; actually detected length {} for Bzip2 QOI Image data",
                    expected_size,
                    bz2_stream_length + header_size,
                );
            }
        }

        // Read entire image (excluding bz2 header) to byte array
        reader.cur_pos = start_position + header_size;
        let raw_image_data: &[u8] = reader
            .read_bytes_dyn(bz2_stream_length)
            .map_err(|e| format!("Trying to read Bzip2 Stream of Bz2 Qoi Image {e}"))?;

        let u16_from = match reader.endianness {
            Endianness::Little => u16::from_le_bytes,
            Endianness::Big => u16::from_be_bytes,
        };
        let width: u16 = u16_from((&header[4..6]).try_into().unwrap());
        let height: u16 = u16_from((&header[6..8]).try_into().unwrap());
        let header = BZip2QoiHeader { width, height, uncompressed_size };
        let image: GMImage = GMImage::from_bz2_qoi(raw_image_data.to_vec(), header);
        Ok(image)
    } else if header.starts_with(MAGIC_QOI_HEADER) {
        // Parse QOI (untested)
        let data_size = reader.read_usize()?;
        reader.cur_pos = start_position;
        let raw_image_data: Vec<u8> = reader
            .read_bytes_dyn(data_size + 12)
            .map_err(|e| format!("Trying to read raw image data {e}"))?
            .to_vec();
        let image: GMImage = GMImage::from_qoi(raw_image_data);
        Ok(image)
    } else {
        let dump: String = hexdump(&header, ..)?;
        bail!("Invalid image header [{dump}] while parsing texture at position {start_position} in chunk 'TXTR'");
    }
}

#[derive(Debug, Clone)]
pub struct BZip2QoiHeader {
    width: u16,
    height: u16,
    /// Present in 2022.5+
    uncompressed_size: Option<u32>,
}

#[derive(Debug, Clone)]
pub enum GMImage {
    DynImg(DynamicImage),
    Png(Vec<u8>),
    Bz2Qoi(Vec<u8>, BZip2QoiHeader),
    Qoi(Vec<u8>),
    /// Only temporarily used when parsing.
    NotYetDeserialized(u32),
}
impl GMImage {
    pub fn from_dynamic_image(dyn_img: DynamicImage) -> Self {
        Self::DynImg(dyn_img)
    }

    pub fn from_png(raw_png_data: Vec<u8>) -> Self {
        Self::Png(raw_png_data)
    }

    pub fn from_bz2_qoi(raw_bz2_qoi_data: Vec<u8>, header: BZip2QoiHeader) -> Self {
        Self::Bz2Qoi(raw_bz2_qoi_data, header)
    }

    pub fn from_qoi(raw_qoi_data: Vec<u8>) -> Self {
        Self::Qoi(raw_qoi_data)
    }

    pub fn to_dynamic_image(&self) -> Result<Cow<DynamicImage>> {
        Ok(match self {
            GMImage::DynImg(dyn_img) => Cow::Borrowed(dyn_img),
            GMImage::Png(raw_png_data) => Cow::Owned(Self::decode_png(&raw_png_data)?),
            GMImage::Bz2Qoi(raw_bz2_qoi_data, _) => Cow::Owned(Self::decode_bz2_qoi(&raw_bz2_qoi_data)?),
            GMImage::Qoi(raw_qoi_data) => Cow::Owned(Self::decode_qoi(&raw_qoi_data)?),
            GMImage::NotYetDeserialized(_) => bail!("Image not deserialized"),
        })
    }

    pub fn into_dynamic_image(self) -> Result<Self> {
        Ok(GMImage::DynImg(match self {
            GMImage::DynImg(dyn_img) => dyn_img,
            GMImage::Png(raw_png_data) => Self::decode_png(&raw_png_data)?,
            GMImage::Bz2Qoi(raw_bz2_qoi_data, _) => Self::decode_bz2_qoi(&raw_bz2_qoi_data)?,
            GMImage::Qoi(raw_qoi_data) => Self::decode_qoi(&raw_qoi_data)?,
            GMImage::NotYetDeserialized(_) => bail!("Image not deserialized"),
        }))
    }

    fn decode_png(raw_png_data: &[u8]) -> Result<DynamicImage> {
        image::load_from_memory_with_format(raw_png_data, ImageFormat::Png).context("Could not parse PNG")
    }

    fn decode_bz2_qoi(raw_bz2_qoi_data: &[u8]) -> Result<DynamicImage> {
        let mut decoder: BzDecoder<&[u8]> = BzDecoder::new(raw_bz2_qoi_data);
        let mut decompressed_data: Vec<u8> = Vec::new();
        decoder
            .read_to_end(&mut decompressed_data)
            .context("Could not decode Bzip2 stream for BzQoi image")?;
        let image = qoi::get_image_from_bytes(&decompressed_data).context("Could not decode Qoi image")?;
        Ok(image)
    }

    fn decode_qoi(raw_qoi_data: &[u8]) -> Result<DynamicImage> {
        let image = qoi::get_image_from_bytes(&raw_qoi_data).context("Could not decode Qoi image")?;
        Ok(image)
    }

    pub fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        match self {
            GMImage::DynImg(dyn_img) => {
                let mut png_data: Vec<u8> = Vec::new();
                dyn_img
                    .write_to(&mut Cursor::new(&mut png_data), ImageFormat::Png)
                    .context("Could not write PNG image data")?;
                builder.write_bytes(&png_data);
            }
            GMImage::Png(raw_png_data) => builder.write_bytes(&raw_png_data),
            GMImage::Bz2Qoi(raw_bz2_qoi_data, header) => {
                builder.write_bytes(MAGIC_BZ2_QOI_HEADER);
                builder.write_u16(header.width);
                builder.write_u16(header.height);
                header
                    .uncompressed_size
                    .serialize_if_gm_ver(builder, "Uncompressed data size", (2022, 5))?;
                builder.write_bytes(&raw_bz2_qoi_data);
            }
            GMImage::Qoi(raw_qoi_data) => builder.write_bytes(raw_qoi_data),
            GMImage::NotYetDeserialized(_) => bail!("Image not deserialized"),
        }
        Ok(())
    }
}

impl PartialEq for GMImage {
    fn eq(&self, other: &Self) -> bool {
        let img1 = match self.to_dynamic_image() {
            Ok(dyn_img) => dyn_img,
            Err(_) => return false,
        };
        let img2 = match other.to_dynamic_image() {
            Ok(dyn_img) => dyn_img,
            Err(_) => return false,
        };
        img1.eq(&img2)
    }
}

fn find_end_of_bz2_stream(reader: &mut DataReader, max_end_of_stream_pos: usize) -> Result<usize> {
    const MAX_CHUNK_SIZE: usize = 256;
    // Read backwards from the max end of stream position, in up to 256-byte chunks.
    // We want to find the end of nonzero data.

    let stream_start_position: usize = reader.cur_pos;
    let mut chunk_start_position: usize = max(stream_start_position, max_end_of_stream_pos - MAX_CHUNK_SIZE);
    let chunk_size: usize = max_end_of_stream_pos - chunk_start_position;
    loop {
        reader.cur_pos = chunk_start_position;
        let chunk_data: &[u8] = reader
            .read_bytes_dyn(chunk_size)
            .map_err(|e| format!("Trying to read BZip2 stream chunk {e}"))?;
        reader.cur_pos += chunk_size;

        // Find first nonzero byte at end of stream
        let mut position: isize = chunk_size as isize - 1;
        while position >= 0 && chunk_data[position as usize] == 0 {
            position -= 1;
        }

        // If we're at nonzero data, then invoke search for footer magic
        if position >= 0 && chunk_data[position as usize] != 0 {
            let end_data_position: isize = chunk_start_position as isize + position + 1;
            return Ok(find_end_of_bz2_search(reader, end_data_position as usize)?);
        }

        // Move backwards to next chunk
        chunk_start_position = max(stream_start_position, chunk_start_position - MAX_CHUNK_SIZE);
        if chunk_start_position <= stream_start_position {
            bail!("Failed to find nonzero data while trying to find end of bz2 stream");
        }
    }
}

fn find_end_of_bz2_search(reader: &mut DataReader, end_data_position: usize) -> Result<usize> {
    const MAGIC_BZ2_FOOTER: [u8; 6] = [0x17, 0x72, 0x45, 0x38, 0x50, 0x90];
    const BUFFER_LENGTH: usize = 16;

    let start_position: usize = end_data_position - BUFFER_LENGTH;
    if start_position >= reader.chunk.end_pos {
        bail!("Start position out of bounds while searching for end of BZip2 stream");
    }

    // Read 16 bytes from the end of the BZ2 stream
    reader.cur_pos = start_position;
    let data: [u8; BUFFER_LENGTH] = reader
        .read_bytes_const()
        .cloned()
        .map_err(|e| format!("Trying to read Bzip2 stream data {e}"))?;
    // FIXME: if this read fails due to overflow; implement saturating logic like in utmt

    // Start searching for magic, bit by bit (it is not always byte-aligned)
    let mut search_start_position: isize = BUFFER_LENGTH as isize - 1;
    let mut search_start_bit_position: u8 = 0;

    while search_start_position >= 0 {
        // Perform search starting from the current search start position
        let mut found_match: bool = false;
        let mut bit_position: u8 = search_start_bit_position;
        let mut search_position: isize = search_start_position;
        let mut magic_bit_position: i32 = 0;
        let mut magic_position: isize = MAGIC_BZ2_FOOTER.len() as isize - 1;

        while search_position >= 0 {
            // Get bits at search position and corresponding magic position
            let current_byte: u8 = data[search_position as usize];
            let magic_byte: u8 = MAGIC_BZ2_FOOTER[magic_position as usize];

            let current_bit: bool = (current_byte & (1 << bit_position)) != 0;
            let magic_current_bit: bool = (magic_byte & (1 << magic_bit_position)) != 0;

            // If bits mismatch, terminate the current search
            if current_bit != magic_current_bit {
                break;
            }

            // Found a matching bit; progress magic position to next bit
            magic_bit_position += 1;
            if magic_bit_position >= 8 {
                magic_bit_position = 0;
                magic_position -= 1;
            }

            // If we reached the end of the magic, then we successfully found a full match!
            if magic_position < 0 {
                found_match = true;
                break;
            }

            // We didn't find a full match yet, so we also need to progress our search position to the next bit
            bit_position += 1;
            if bit_position >= 8 {
                bit_position = 0;
                search_position -= 1;
            }
        }

        if found_match {
            const FOOTER_BYTE_LENGTH: usize = 10;
            let mut end_of_bz2_stream_position = (search_position + FOOTER_BYTE_LENGTH as isize) as usize;

            if bit_position != 7 {
                // BZip2 footer started partway through a byte, and so it will end partway through the last byte.
                // By the BZip2 specification, the unused bits of the last byte are essentially padding.
                end_of_bz2_stream_position += 1;
            }

            return Ok(start_position + end_of_bz2_stream_position);
        }

        // Current search failed to make a full match, so progress to next bit, to search starting from there
        search_start_bit_position += 1;
        if search_start_bit_position >= 8 {
            search_start_bit_position = 0;
            search_start_position -= 1;
        }
    }

    bail!("Failed to find BZip2 footer magic");
}

fn read_u32_be(reader: &mut DataReader, context: &str) -> Result<u32> {
    Ok(reader
        .read_bytes_const()
        .cloned()
        .map(u32::from_be_bytes)
        .map_err(|e| format!("{context} {e}"))?)
}
