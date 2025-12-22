use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text {
    pub text: String,
    pub line_wrapping: bool,
    pub alignment_v: i8,
    pub alignment_h: i8,
    pub font_index: i32,
}

impl GMElement for Text {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let text: String = reader.read_gm_string()?;
        let line_wrapping = reader.read_bool32()?;
        let alignment = reader.read_i32()?;
        let font_index = reader.read_i32()?;
        Ok(Self {
            text,
            line_wrapping,
            alignment_v: ((alignment >> 8) & 0xFF) as i8,
            alignment_h: (alignment & 0xFF) as i8,
            font_index,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.text);
        builder.write_bool32(self.line_wrapping);
        builder.write_i32(i32::from(self.alignment_v) << 8 | i32::from(self.alignment_h));
        log::warn!(
            "Writing raw Font index {} for Text Keyframe of Sequence",
            self.font_index
        );
        builder.write_i32(self.font_index); // TODO no idea what this is but shouldn't it be a GMRef<GMFont> instead of an i32?
        Ok(())
    }
}
