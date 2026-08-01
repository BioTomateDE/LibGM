// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::GMVersion;
use crate::wad::elem::GMElement;
use crate::wad::elem::font::kerning::Kerning;
use crate::wad::load::reader::DataReader;
use crate::wad::save::writer::DataWriter;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Glyph {
    /// The character this glyph represents.
    pub character: Option<char>,

    /// The x position in the `GMFont.texture` where the glyph can be found.
    pub x: u16,

    /// The y position in the `GMFont.texture` where the glyph can be found.
    pub y: u16,

    /// The width of the glyph in pixels.
    pub width: u16,

    /// The height of the glyph in pixels.
    pub height: u16,

    /// The number of pixels to shift right when advancing to the next
    /// character.
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
        if reader.version >= GMVersion::GM2024_11 {
            let unknown_always_zero = reader.read_i16()?;
            reader.assert_int(unknown_always_zero, 0, "Unknown Always Zero")?;
        }
        let kernings: Vec<Kerning> = reader.read_simple_list_short()?;

        Ok(Self { character, x, y, width, height, shift_modifier, offset, kernings })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        if let Some(char) = self.character {
            let codepoint: u32 = char.into();
            writer.write_u16(codepoint as u16);
        } else {
            writer.write_u16(0);
        }
        writer.write_u16(self.x);
        writer.write_u16(self.y);
        writer.write_u16(self.width);
        writer.write_u16(self.height);
        writer.write_i16(self.shift_modifier);
        writer.write_i16(self.offset);
        if writer.version() >= GMVersion::GM2024_11 {
            writer.write_u16(0); // UnknownAlwaysZero
        }
        writer.write_simple_list_short(&self.kernings)?;
        Ok(())
    }
}
