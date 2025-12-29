pub mod fill;
pub mod line;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
    util::init::vec_with_capacity,
};

#[derive(Debug, Clone, PartialEq)]
pub struct StyleGroup<T> {
    pub fill_styles: Vec<fill::Data>,
    pub line_styles: Vec<line::Data>,
    pub subshapes: Vec<T>,
}
impl<T: GMElement> GMElement for StyleGroup<T> {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let fill_data_count = reader.read_count("YYSWF Style Group Fill Data")?;
        let line_style_count = reader.read_count("YYSWF Style Group Line Style")?;
        let subshape_count = reader.read_count("YYSWF Style Group Subshape")?;

        let mut fill_styles: Vec<fill::Data> = vec_with_capacity(fill_data_count)?;
        for _ in 0..fill_data_count {
            fill_styles.push(fill::Data::deserialize(reader)?);
        }

        let mut line_styles: Vec<line::Data> = vec_with_capacity(line_style_count)?;
        for _ in 0..line_style_count {
            line_styles.push(line::Data::deserialize(reader)?);
        }

        let mut subshapes: Vec<T> = vec_with_capacity(subshape_count)?;
        for _ in 0..subshape_count {
            subshapes.push(T::deserialize(reader)?);
        }

        Ok(Self { fill_styles, line_styles, subshapes })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_usize(self.fill_styles.len())?;
        builder.write_usize(self.line_styles.len())?;
        builder.write_usize(self.subshapes.len())?;
        for fill_data in &self.fill_styles {
            fill_data.serialize(builder)?;
        }
        for line_data in &self.line_styles {
            line_data.serialize(builder)?;
        }
        for subshape in &self.subshapes {
            subshape.serialize(builder)?;
        }
        Ok(())
    }
}
