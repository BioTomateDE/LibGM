use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMElement, font::kerning::Kerning},
        serialize::builder::DataBuilder,
    },
    prelude::*,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Glyph {
    /// The character this glyph represents.
    pub character: Option<char>,

    /// The x position in the [`GMFont`].`texture` where the glyph can be found.
    pub x: u16,

    /// The y position in the [`GMFont`].`texture` where the glyph can be found.
    pub y: u16,

    /// The width of the glyph in pixels.
    pub width: u16,

    /// The height of the glyph in pixels.
    pub height: u16,

    /// The number of pixels to shift right when advancing to the next character.
    pub shift_modifier: i16,

    /// The number of pixels to horizontally offset the rendering of this glyph.
    pub offset: i16,

    /// The kerning for each glyph.
    pub kernings: Vec<Kerning>,
}

impl GMElement for Glyph {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let character = reader.read_u16()?;
        let character: Option<char> = if character == 0 {
            None
        } else {
            Some(char::from_u32(character.into()).ok_or_else(|| {
                format!("Invalid UTF-8 character with code point {character} (0x{character:04X})")
            })?)
        };
        let x = reader.read_u16()?;
        let y = reader.read_u16()?;
        let width = reader.read_u16()?;
        let height = reader.read_u16()?;
        let shift_modifier = reader.read_i16()?;
        let offset = reader.read_i16()?; // Potential assumption according to utmt
        if reader.general_info.is_version_at_least((2024, 11)) {
            let unknown_always_zero = reader.read_i16()?;
            reader.assert_int(unknown_always_zero, 0, "Unknown Always Zero")?;
        }
        let kernings: Vec<Kerning> = reader.read_simple_list_short()?;

        Ok(Self {
            character,
            x,
            y,
            width,
            height,
            shift_modifier,
            offset,
            kernings,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        if let Some(char) = self.character {
            let codepoint: u32 = char.into();
            builder.write_u16(codepoint as u16);
        } else {
            builder.write_u16(0);
        }
        builder.write_u16(self.x);
        builder.write_u16(self.y);
        builder.write_u16(self.width);
        builder.write_u16(self.height);
        builder.write_i16(self.shift_modifier);
        builder.write_i16(self.offset);
        if builder.is_gm_version_at_least((2024, 11)) {
            builder.write_u16(0); // UnknownAlwaysZero
        }
        builder.write_simple_list_short(&self.kernings)?;
        Ok(())
    }
}
