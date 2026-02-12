mod bz2;
mod png;
mod qoi;

use std::{borrow::Cow, fmt};

pub use bz2::BZip2QoiHeader;
use image::DynamicImage;

use crate::{
    gamemaker::{
        elements::embedded_texture::BZ2_QOI_HEADER,
        serialize::{builder::DataBuilder, traits::GMSerializeIfVersion},
    },
    prelude::*,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    /// A deserialized image stored as a [`DynamicImage`].
    Dyn,

    /// The PNG image format (Portable Network Graphics).
    Png,

    /// GameMaker's custom QOI image format (Quite Ok Image).
    Qoi,

    /// GameMaker's custom QOI image format, compressed with [BZip2](https://en.wikipedia.org/wiki/Bzip2).
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

    pub fn to_dynamic_image(&'_ self) -> Result<Cow<'_, DynamicImage>> {
        Ok(match &self.0 {
            Img::Dyn(dyn_img) => Cow::Borrowed(dyn_img),
            Img::Png(raw) => Cow::Owned(png::decode(raw)?),
            Img::Qoi(raw) => Cow::Owned(qoi::decode(raw)?),
            Img::Bz2Qoi(raw, _) => Cow::Owned(bz2::decode_image(raw)?),
        })
    }

    /// The image format of the underlying stored image data.
    #[must_use]
    pub const fn format(&self) -> Format {
        match self.0 {
            Img::Dyn(_) => Format::Dyn,
            Img::Png(_) => Format::Png,
            Img::Qoi(_) => Format::Qoi,
            Img::Bz2Qoi(_, _) => Format::Bz2Qoi,
        }
    }

    /// Changes the format of the underlying stored image data.
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
    /// ```no_run
    /// let mut gm_image = texture_page.image.as_mut().unwrap();
    /// gm_image.change_format(Format::Dyn)?; // Add this
    /// gm_image.change_format(Format::Bz2Qoi)?; // (or whatever format you want)
    /// ```
    pub fn change_format(&mut self, format: Format) -> Result<()> {
        let old = self.format();
        if old == format {
            return Ok(());
        }
        self.change_format_(format)
            .with_context(|| format!("converting GMImage from {old} to {format}"))
    }

    fn change_format_(&mut self, format: Format) -> Result<()> {
        // Special case when converting between Bz2Qoi and Qoi (optimisation)
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

        let dyn_img = self.to_dynamic_image()?;

        let new_image = match format {
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

    pub(super) fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        match &self.0 {
            Img::Dyn(dyn_img) => write_dyn_img(dyn_img, builder)?,
            Img::Png(raw_png_data) => builder.write_bytes(raw_png_data),
            Img::Qoi(raw_qoi_data) => builder.write_bytes(raw_qoi_data),
            Img::Bz2Qoi(raw_bz2_qoi_data, header) => {
                write_bz2qoi_header(header, builder)?;
                builder.write_bytes(raw_bz2_qoi_data);
            },
        }
        Ok(())
    }
}

// TODO(weak): this is still kind of ass
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
    // PNG is preferred
    if cfg!(feature = "png-image") {
        let data: Vec<u8> = png::encode(dyn_img)?;
        builder.write_bytes(&data);
        return Ok(());
    }

    // If PNG is disabled, use Bz2Qoi
    if cfg!(feature = "bzip2-image") {
        let (data, header) = bz2::encode_image(dyn_img)?;
        write_bz2qoi_header(&header, builder)?;
        builder.write_bytes(&data);
        return Ok(());
    }

    // Fallback to raw QOI
    qoi::build(dyn_img, builder)?;

    Ok(())
}

fn write_bz2qoi_header(header: &BZip2QoiHeader, builder: &mut DataBuilder) -> Result<()> {
    builder.write_bytes(BZ2_QOI_HEADER);
    builder.write_u16(header.width);
    builder.write_u16(header.height);
    header
        .uncompressed_size
        .serialize_if_gm_ver(builder, "Uncompressed data size", (2022, 5))?;
    Ok(())
}
