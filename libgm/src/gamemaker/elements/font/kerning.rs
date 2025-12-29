use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Kerning {
    /// The preceding character.
    pub character: char,
    /// An amount of pixels to add to the existing [`GMFontGlyph`].`shift_modifier`.
    pub shift_modifier: i16,
}

impl GMElement for Kerning {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let character = reader.read_u16()?;
        if character == 0 {
            bail!("Character not set (code point is zero)");
        }
        let character: char = char::from_u32(character.into()).ok_or_else(|| {
            format!("Invalid UTF-8 character with code point {character} (0x{character:04X})")
        })?;
        let shift_modifier = reader.read_i16()?;
        Ok(Self { character, shift_modifier })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_u16(u32::from(self.character) as u16);
        builder.write_i16(self.shift_modifier);
        Ok(())
    }
}
