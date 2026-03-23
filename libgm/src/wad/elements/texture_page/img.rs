mod bz2;
mod png;
mod qoi;

use std::{borrow::Cow, fmt};

pub use bz2::BZip2QoiHeader;
use image::DynamicImage;

use crate::{
    prelude::*,
    wad::{elements::texture_page::BZ2_QOI_HEADER, serialize::builder::DataBuilder},
};

/// An image format indicating how the underlying data of a [`GMImage`] is stored.
///
/// This can be changed (using decoding/encoding algorithms) using [`GMImage::change_format`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    /// A deserialized image stored as a [`DynamicImage`].
    Dyn,

    /// The PNG image format (Portable Network Graphics).
    Png,

    /// The [QOI](https://qoiformat.org/) image format (Quite Ok Image).
    ///
    /// This is available since GM 2022.1.
    /// Source: [Wikipedia](https://en.wikipedia.org/wiki/QOI_(image_format)#:~:text=The%20game%20engine%20GameMaker%20has>).
    ///
    /// GameMaker's QOI implementation has slight variations from the official spec.
    /// For example, the header is stored differently.
    ///
    /// Official Header:
    /// * 4 bytes - Magic bytes "QOIF"
    /// * 4 bytes - Image width in pixels (big endian)
    /// * 4 bytes - Image height in pixels (big endian)
    /// * 1 byte - Channel count (3 = RGB, 4 = RGBA)
    /// * 1 byte - Colorspace (0 = sRGB with linear alpha, 1 = all channels linear)
    ///
    /// GameMaker Header:
    /// * 4 bytes - Magic bytes "FIOQ" (reversed because of little endian)
    /// * 2 bytes - Image width in pixels (little endian)
    /// * 2 bytes - Image height in pixels (little endian)
    /// * 4 bytes - Encoded byte length (excluding header)
    Qoi,

    /// GameMaker's custom QOI image format, compressed with [BZip2](https://en.wikipedia.org/wiki/Bzip2).
    ///
    /// For more information, see [`Format::Qoi`].
    Bz2Qoi,
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string: &str = match self {
            Self::Dyn => "Dynamic Image",
            Self::Png => "PNG",
            Self::Qoi => "QOI",
            Self::Bz2Qoi => "BZip2 QOI",
        };
        f.write_str(string)
    }
}

#[derive(Debug, Clone)]
pub struct GMImage(Img);

impl GMImage {
    /// Creates a new [`GMImage`] from the specified [`DynamicImage`], consuming it.
    #[must_use]
    pub const fn from_dynamic_image(dyn_img: DynamicImage) -> Self {
        Self(Img::Dyn(dyn_img))
    }

    #[must_use]
    pub(super) const fn from_png(raw_png_data: Vec<u8>) -> Self {
        Self(Img::Png(raw_png_data))
    }

    #[must_use]
    pub(super) const fn from_qoi(raw_qoi_data: Vec<u8>) -> Self {
        Self(Img::Qoi(raw_qoi_data))
    }

    #[must_use]
    pub(super) const fn from_bz2_qoi(raw_bz2_qoi_data: Vec<u8>, header: BZip2QoiHeader) -> Self {
        Self(Img::Bz2Qoi(raw_bz2_qoi_data, header))
    }

    /// Converts this image to a [`DynamicImage`].
    ///
    /// This is a no-op if the underlying data is already a [`DynamicImage`].
    ///
    /// Note that this does *not* change the format of `self`.
    pub fn to_dynamic_image(&'_ self) -> Result<Cow<'_, DynamicImage>> {
        let image: DynamicImage = match &self.0 {
            Img::Dyn(dyn_img) => return Ok(Cow::Borrowed(dyn_img)),
            Img::Png(raw) => png::decode(raw).context("converting PNG image to DynamicImage")?,
            Img::Qoi(raw) => qoi::decode(raw).context("converting QOI image to DynamicImage")?,
            Img::Bz2Qoi(raw, _) => {
                bz2::decode_image(raw).context("converting Bz2Qoi image to DynamicImage")?
            },
        };
        Ok(Cow::Owned(image))
    }

    /// The Image [`Format`] of the underlying stored image data.
    #[must_use]
    pub const fn format(&self) -> Format {
        match self.0 {
            Img::Dyn(_) => Format::Dyn,
            Img::Png(_) => Format::Png,
            Img::Qoi(_) => Format::Qoi,
            Img::Bz2Qoi(_, _) => Format::Bz2Qoi,
        }
    }

    /// Whether the underlying image data ([`Format`]) is a [`DynamicImage`].
    #[must_use]
    pub const fn is_dynamic_image(&self) -> bool {
        // TODO(const-hack): const PartialEq not yet supported
        matches!(self.format(), Format::Dyn)
    }

    /// Changes the format of the underlying stored image data.
    ///
    /// Returns `Ok(true)` if the format was actually changed,
    /// `Ok(false)` if the format is the same and `Err(...)` if the image (de)serialization failed.
    ///
    /// It will deserialize the data to a [`DynamicImage`], if needed.
    /// Then, the [`DynamicImage`] is serialized to the desired format again.
    ///
    /// No operation will be done if the image is already in the specified format.
    /// For conversions between `Bz2Qoi` and `Qoi`, the intermediate `DynamicImage` step is
    /// skipped.
    ///
    /// In order to force intermediate deserialization to [`DynamicImage`]
    /// (to catch invalid image data, for example), you can use code like this:
    ///
    /// ```ignore
    /// let mut gm_image = texture_page.image.as_mut().ok_or("No image data in texture page")?;
    /// gm_image.change_format(Format::Dyn)?; // Add this
    /// gm_image.change_format(Format::Bz2Qoi)?; // (or whatever format you want)
    /// ```
    pub fn change_format(&mut self, format: Format) -> Result<bool> {
        let old = self.format();
        if old == format {
            return Ok(false);
        }
        self.change_format_(format)
            .with_context(|| format!("converting GMImage from {old} to {format}"))?;
        Ok(true)
    }

    /// Turns the underlying data of this [`GMImage`] into a [`DynamicImage`].
    ///
    /// This will deserialize PNG/QOI data or do nothing if the image is already stored as a [`DynamicImage`].
    ///
    /// Returns `Ok(true)` if the format was actually changed,
    /// `Ok(false)` if it was already a [`DynamicImage`] and `Err(...)` if the image deserialization failed.
    ///
    /// For more information, see [`GMImage::change_format`].
    pub fn deserialize(&mut self) -> Result<bool> {
        self.change_format(Format::Dyn)
    }

    fn change_format_(&mut self, format: Format) -> Result<()> {
        // Special case when converting between Bz2Qoi and Qoi (optimization)
        match (&self.0, format) {
            (Img::Qoi(raw_data), Format::Bz2Qoi) => {
                let qoi_header = qoi::read_header(raw_data)?;
                let size = Some(raw_data.len() as u32);
                let bz2_header = BZip2QoiHeader::new(qoi_header.width, qoi_header.height, size);
                let data: Vec<u8> = bz2::compress(raw_data)?;
                self.0 = Img::Bz2Qoi(data, bz2_header);
                return Ok(());
            },
            (Img::Bz2Qoi(raw_data, _), Format::Qoi) => {
                let data: Vec<u8> = bz2::decompress(raw_data)?;
                self.0 = Img::Qoi(data);
                return Ok(());
            },
            _ => {},
        }

        // Normal conversion. First convert to DynamicImage and then re-encode in the desired format.
        let dyn_img = self.to_dynamic_image()?;

        let new_image = match format {
            // This `into_owned()` never actually clones:
            // to_dynamic_image returns Cow::Owned only if it was not a DynamicImage before
            // which would be impossible (since Dyn -> Dyn is skipped out by change_format)
            Format::Dyn => Img::Dyn(dyn_img.into_owned()),
            Format::Png => Img::Png(png::encode(&dyn_img)?),
            Format::Qoi => Img::Qoi(qoi::encode(&dyn_img)?),
            Format::Bz2Qoi => {
                let (data, header) = bz2::encode_image(&dyn_img)?;
                Img::Bz2Qoi(data, header)
            },
        };

        self.0 = new_image;
        Ok(())
    }

    /// Tries to optimize this image's memory footprint by shrinking
    /// underlying buffers to their needed size.
    ///
    /// Returns the number of bytes freed (can be 0).
    ///
    /// This function takes up CPU power and should not be called frequently, if at all.
    ///
    /// If you're looking to optimize [`GMData`] memory in general, visit [`GMData::optimize_memory`].
    pub(crate) fn optimize_memory(&mut self) -> usize {
        // not public for now. make gh issue if u want this
        fn shrink(buffer: &mut Vec<u8>) -> usize {
            let before = buffer.capacity();
            buffer.shrink_to_fit();
            let after = buffer.capacity();
            before - after
        }

        match &mut self.0 {
            Img::Dyn(_) => 0, // idk if u can shrink this
            Img::Png(buffer) | Img::Qoi(buffer) | Img::Bz2Qoi(buffer, _) => shrink(buffer),
        }
    }

    pub(super) fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        let is_qoi = matches!(self.0, Img::Qoi(_) | Img::Bz2Qoi(_, _));
        let is_qoi_eligible = builder.is_version_at_least((2022, 2));
        if is_qoi && !is_qoi_eligible {
            bail!("Cannot serialize QOI images before GM 2022.2");
        }

        match &self.0 {
            Img::Dyn(dyn_img) => {
                write_dyn_img(dyn_img, builder).context("serializing DynamicImage")?;
            },
            Img::Png(raw_png_data) => builder.write_bytes(raw_png_data),
            Img::Qoi(raw_qoi_data) => builder.write_bytes(raw_qoi_data),
            Img::Bz2Qoi(raw_bz2_qoi_data, header) => {
                write_bz2qoi_header(header, builder).context("writing Bz2Qoi image header")?;
                builder.write_bytes(raw_bz2_qoi_data);
            },
        }
        Ok(())
    }
}

// TODO(weak): this is still kind of ass
/// WARNING: I do not recommend using this trait to compare [`GMImage`]s.
/// This only exists to satisfy the [`GMChunk`] bound of [`PartialEq`] (proc macro issue lol).
///
/// Instead, deserialize the images first and properly handle errors.
impl PartialEq for GMImage {
    fn eq(&self, other: &Self) -> bool {
        let Ok(img1) = self.to_dynamic_image() else {
            log::warn!("Deserialization failed while comparing GMImage");
            return false;
        };
        let Ok(img2) = other.to_dynamic_image() else {
            log::warn!("Deserialization failed while comparing GMImage");
            return false;
        };
        img1 == img2
    }
}

#[derive(Clone)]
enum Img {
    Dyn(DynamicImage),
    Png(Vec<u8>),
    Qoi(Vec<u8>),
    Bz2Qoi(Vec<u8>, BZip2QoiHeader),
}

impl fmt::Debug for Img {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dyn(_) => f.write_str("Dyn"),
            Self::Png(_) => f.write_str("Png"),
            Self::Qoi(_) => f.write_str("Qoi"),
            Self::Bz2Qoi(_, header) => f.debug_tuple("Bz2Qoi").field(header).finish(),
        }
    }
}

fn write_dyn_img(dyn_img: &DynamicImage, builder: &mut DataBuilder) -> Result<()> {
    // Use QOI if supported.
    if builder.is_version_at_least((2022, 1)) {
        qoi::build(dyn_img, builder).context("serializing DynamicImage as QOI")?;
        return Ok(());
    }

    if !cfg!(feature = "png-image") {
        bail!("Crate feature `png-image` is disabled and the game is too old to use QOI images.");
    }

    let data: Vec<u8> = png::encode(dyn_img)?;
    builder.write_bytes(&data);
    Ok(())
}

fn write_bz2qoi_header(header: &BZip2QoiHeader, builder: &mut DataBuilder) -> Result<()> {
    builder.write_bytes(BZ2_QOI_HEADER);
    builder.write_u16(header.width);
    builder.write_u16(header.height);
    builder.write_if_ver(&header.uncompressed_size, "Uncompressed Size", (2022, 5))?;
    Ok(())
}
