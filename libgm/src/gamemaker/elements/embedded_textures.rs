use crate::gamemaker::data::Endianness;
use crate::gamemaker::deserialize::reader::DataReader;
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::qoi;
use crate::gamemaker::serialize::builder::DataBuilder;
use crate::gamemaker::serialize::traits::GMSerializeIfVersion;
use crate::prelude::*;
use crate::util::fmt::hexdump;
use bzip2::read::BzDecoder;
use image;
use image::{DynamicImage, ImageFormat};
use std::borrow::Cow;
use std::cmp::max;
use std::io::{Cursor, Read};
use std::ops::{Deref, DerefMut};

pub(crate) const MAGIC_PNG_HEADER: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
pub(crate) const MAGIC_BZ2_QOI_HEADER: &[u8; 4] = b"2zoq";
pub(crate) const MAGIC_QOI_HEADER: &[u8; 4] = b"fioq";

#[derive(Debug, Clone, Default)]
pub struct GMEmbeddedTextures {
    pub texture_pages: Vec<GMEmbeddedTexture>,
    pub exists: bool,
}

impl Deref for GMEmbeddedTextures {
    type Target = Vec<GMEmbeddedTexture>;
    fn deref(&self) -> &Self::Target {
        &self.texture_pages
    }
}

impl DerefMut for GMEmbeddedTextures {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.texture_pages
    }
}

impl GMChunkElement for GMEmbeddedTextures {
    const NAME: &'static str = "TXTR";
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMEmbeddedTextures {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let mut texture_pages: Vec<GMEmbeddedTexture> = reader.read_pointer_list()?;
        for i in 0..texture_pages.len() {
            // Find next element start position
            let mut max_stream_end_pos = reader.chunk.end_pos;
            for texture_page in &texture_pages[i + 1..] {
                let Some(ref img) = texture_page.image else {
                    continue;
                };
                let &GMImage::NotYetDeserialized(blob_pos) = img else {
                    bail!("GMImage enum variant is somehow not `NotYetDeserialized`");
                };
                max_stream_end_pos = blob_pos;
                break;
            }

            let texture_page: &mut GMEmbeddedTexture = &mut texture_pages[i];
            let Some(ref mut gm_image) = texture_page.image else {
                continue; // Texture is external
            };
            let GMImage::NotYetDeserialized(blob_position) = gm_image else {
                bail!("GMImage enum variant is somehow not `NotYetDeserialized`");
            };
            reader.cur_pos = *blob_position;
            *gm_image = read_raw_texture(reader, max_stream_end_pos, texture_page.texture_block_size)?;
        }

        reader.align(4)?;
        Ok(Self { texture_pages, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
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
            let Some(img) = &texture_page.image else {
                continue;
            };
            builder.align(0x80);
            builder.resolve_pointer(&texture_page.image)?;
            let start_pos: usize = builder.len();
            img.serialize(builder)?;
            if builder.is_gm_version_at_least((2022, 3)) {
                let length: usize = builder.len() - start_pos;
                builder.overwrite_usize(length, texture_block_size_placeholders[i])?
            }
        }

        builder.align(4);
        Ok(())
    }
}

/// An embedded texture entry in the data file.
#[derive(Debug, Clone, PartialEq)]
#[repr(C)] // Needs explicit layout so memory addresses for gm pointers don't collide
pub struct GMEmbeddedTexture {
    /// not sure what `scaled` actually is
    pub scaled: u32,

    /// The amount of generated MipMap levels. Present in 2.0.6+
    pub generated_mips: Option<u32>,

    /// Size of the texture attached to this texture page in bytes.
    /// Present in 2022.3+, as long as the texture is not external.
    pub texture_block_size: Option<u32>,

    pub data_2022_9: Option<GMEmbeddedTexture2022_9>,

    /// The texture data in the embedded image.
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
            None // external texture if texture_data_start_pos is zero
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
    /// Width of the texture.
    pub texture_width: u32,
    /// Height of the texture.
    pub texture_height: u32,
    /// Index of the texture in the texture group.
    pub index_in_group: u32,
}

impl GMElement for GMEmbeddedTexture2022_9 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let texture_width = reader.read_u32()?;
        let texture_height = reader.read_u32()?;
        let index_in_group = reader.read_u32()?;
        Ok(Self { texture_width, texture_height, index_in_group })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_u32(self.texture_width);
        builder.write_u32(self.texture_height);
        builder.write_u32(self.index_in_group);
        Ok(())
    }
}

fn read_raw_texture(
    reader: &mut DataReader,
    max_stream_end_pos: u32,
    texture_block_size: Option<u32>,
) -> Result<GMImage> {
    reader.align(0x80)?;
    let header: [u8; 8] = *reader.read_bytes_const().context("reading image header")?;

    let (image, data_length) = if header == MAGIC_PNG_HEADER {
        read_png(reader)?
    } else if header.starts_with(MAGIC_BZ2_QOI_HEADER) {
        read_bz2_qoi(reader, &header, max_stream_end_pos)?
    } else if header.starts_with(MAGIC_QOI_HEADER) {
        read_qoi(reader)?
    } else {
        let dump: String = hexdump(&header, ..)?;
        bail!("Invalid image header [{dump}]");
    };

    if let Some(expected_size) = texture_block_size {
        if expected_size != data_length {
            bail!(
                "Texture Page Entry specified texture block size {expected_size}; \
                actually read image with length {data_length}"
            );
        }
    }

    Ok(image)
}

fn read_png(reader: &mut DataReader) -> Result<(GMImage, u32)> {
    let start_position = reader.cur_pos - 8;
    loop {
        let length: u32 = reader
            .read_bytes_const()
            .cloned()
            .map(u32::from_be_bytes)
            .context("reading PNG chunk length")?;
        let chunk_type: [u8; 4] = reader.read_bytes_const().cloned().context("reading PNG chunk type")?;
        reader.cur_pos += length + 4;
        if &chunk_type == b"IEND" {
            break;
        }
    }

    let data_length = reader.cur_pos - start_position;
    reader.cur_pos = start_position;
    let bytes: &[u8] = reader.read_bytes_dyn(data_length).context("reading PNG image data")?;
    // Png image size checks {~~}
    let image = GMImage::from_png(bytes.to_vec());
    Ok((image, data_length))
}

fn read_bz2_qoi(reader: &mut DataReader, header: &[u8; 8], max_end_of_stream_pos: u32) -> Result<(GMImage, u32)> {
    let start_position = reader.cur_pos - 8;
    let mut header_size = 8;
    let mut uncompressed_size = None;
    if reader.general_info.is_version_at_least((2022, 5)) {
        uncompressed_size = Some(reader.read_u32()?);
        header_size = 12;
    }

    let bz2_stream_end = find_end_of_bz2_stream(reader, max_end_of_stream_pos)?;
    let bz2_stream_length = bz2_stream_end - start_position - header_size;
    let data_length = bz2_stream_length + header_size;

    // Read entire image (excluding bz2 header) to byte array
    reader.cur_pos = start_position + header_size;
    let raw_image_data: &[u8] = reader
        .read_bytes_dyn(bz2_stream_length)
        .context("reading BZip2 Stream of BZip2 QOI Image")?;

    let u16_from = match reader.endianness {
        Endianness::Little => u16::from_le_bytes,
        Endianness::Big => u16::from_be_bytes,
    };
    let width: u16 = u16_from((&header[4..6]).try_into().unwrap());
    let height: u16 = u16_from((&header[6..8]).try_into().unwrap());
    let header = BZip2QoiHeader { width, height, uncompressed_size };
    let image: GMImage = GMImage::from_bz2_qoi(raw_image_data.to_vec(), header);
    Ok((image, data_length))
}

fn read_qoi(reader: &mut DataReader) -> Result<(GMImage, u32)> {
    let start_position = reader.cur_pos - 8;
    let data_length = reader.read_u32()?;
    reader.cur_pos = start_position;
    let raw_image_data: Vec<u8> = reader
        .read_bytes_dyn(data_length + 12)
        .context("reading QOI Image data")?
        .to_vec();
    let image: GMImage = GMImage::from_qoi(raw_image_data);
    Ok((image, data_length))
}

#[derive(Debug, Clone)]
pub struct BZip2QoiHeader {
    width: u16,
    height: u16,
    /// Present in 2022.5+
    uncompressed_size: Option<u32>,
}

/// **This is not an actual GMElement!**
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

    pub(crate) fn from_png(raw_png_data: Vec<u8>) -> Self {
        Self::Png(raw_png_data)
    }

    pub(crate) fn from_bz2_qoi(raw_bz2_qoi_data: Vec<u8>, header: BZip2QoiHeader) -> Self {
        Self::Bz2Qoi(raw_bz2_qoi_data, header)
    }

    pub(crate) fn from_qoi(raw_qoi_data: Vec<u8>) -> Self {
        Self::Qoi(raw_qoi_data)
    }

    pub fn to_dynamic_image(&'_ self) -> Result<Cow<'_, DynamicImage>> {
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
        let image = qoi::deserialize(&decompressed_data).context("Could not decode Qoi image")?;
        Ok(image)
    }

    fn decode_qoi(raw_qoi_data: &[u8]) -> Result<DynamicImage> {
        let image = qoi::deserialize(&raw_qoi_data).context("Could not decode Qoi image")?;
        Ok(image)
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
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
        img1 == img2
    }
}

fn find_end_of_bz2_stream(reader: &mut DataReader, max_end_of_stream_pos: u32) -> Result<u32> {
    const MAX_CHUNK_SIZE: u32 = 256;
    // Read backwards from the max end of stream position, in up to 256-byte chunks.
    // We want to find the end of nonzero data.

    let stream_start_position = reader.cur_pos;
    let mut chunk_start_position = max(stream_start_position, max_end_of_stream_pos - MAX_CHUNK_SIZE);
    let chunk_size = max_end_of_stream_pos - chunk_start_position;
    loop {
        reader.cur_pos = chunk_start_position;
        let chunk_data: &[u8] = reader
            .read_bytes_dyn(chunk_size)
            .context("reading BZip2 stream chunk")?;
        reader.cur_pos += chunk_size;

        // Find first nonzero byte at end of stream
        let mut position = chunk_size as i32 - 1;
        while position >= 0 && chunk_data[position as usize] == 0 {
            position -= 1;
        }

        // If we're at nonzero data, then invoke search for footer magic
        if position >= 0 && chunk_data[position as usize] != 0 {
            let end_data_position = chunk_start_position + position as u32 + 1;
            return Ok(find_end_of_bz2_search(reader, end_data_position)?);
        }

        // Move backwards to next chunk
        chunk_start_position = max(stream_start_position, chunk_start_position - MAX_CHUNK_SIZE);
        if chunk_start_position <= stream_start_position {
            bail!("Failed to find nonzero data while trying to find end of bz2 stream");
        }
    }
}

fn find_end_of_bz2_search(reader: &mut DataReader, end_data_position: u32) -> Result<u32> {
    const MAGIC_BZ2_FOOTER: [u8; 6] = [0x17, 0x72, 0x45, 0x38, 0x50, 0x90];
    const BUFFER_LENGTH: u32 = 16;

    let start_position = end_data_position - BUFFER_LENGTH;
    if start_position >= reader.chunk.end_pos {
        bail!("Start position out of bounds while searching for end of BZip2 stream");
    }

    // Read 16 bytes from the end of the BZ2 stream
    reader.cur_pos = start_position;
    let data: [u8; BUFFER_LENGTH as usize] = reader
        .read_bytes_const()
        .cloned()
        .context("reading BZip2 stream data")?;
    // If this read fails due to overflow; implement saturating logic like in utmt

    // Start searching for magic, bit by bit (it is not always byte-aligned)
    let mut search_start_position = BUFFER_LENGTH as i32 - 1;
    let mut search_start_bit_position: u8 = 0;

    while search_start_position >= 0 {
        // Perform search starting from the current search start position
        let mut found_match: bool = false;
        let mut bit_position: u8 = search_start_bit_position;
        let mut search_position: i32 = search_start_position;
        let mut magic_bit_position: i32 = 0;
        let mut magic_position = MAGIC_BZ2_FOOTER.len() as i8 - 1;

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
            const FOOTER_BYTE_LENGTH: u32 = 10;
            let mut end_of_bz2_stream_position = (search_position + FOOTER_BYTE_LENGTH as i32) as u32;

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
