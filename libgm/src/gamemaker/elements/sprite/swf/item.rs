pub mod bitmap;
pub mod shape;
pub mod subshape;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub id: i32,
    pub item_data: ItemData,
}

impl GMElement for Item {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let item_type = reader.read_i32()?;
        let id = reader.read_i32()?;
        let item_data: ItemData = match item_type {
            1 => ItemData::ItemShape(shape::Data::deserialize(reader)?),
            2 => ItemData::ItemBitmap(bitmap::Data::deserialize(reader)?),
            3 => ItemData::ItemFont,
            4 => ItemData::ItemTextField,
            5 => ItemData::ItemSprite,
            _ => bail!(
                "Invalid YYSWF Item Type {0} 0x{0:08X} at position {1} while parsing Sprite YYSWF Item",
                item_type,
                reader.cur_pos,
            ),
        };
        Ok(Self { id, item_data })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(match &self.item_data {
            ItemData::ItemShape(_) => 1,
            ItemData::ItemBitmap(_) => 2,
            ItemData::ItemFont => 3,
            ItemData::ItemTextField => 4,
            ItemData::ItemSprite => 5,
        });
        builder.write_i32(self.id);
        match &self.item_data {
            ItemData::ItemShape(shape_data) => shape_data.serialize(builder)?,
            ItemData::ItemBitmap(bitmap_data) => bitmap_data.serialize(builder)?,
            ItemData::ItemFont | ItemData::ItemTextField | ItemData::ItemSprite => {},
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemData {
    ItemShape(shape::Data<subshape::Data>),
    ItemBitmap(bitmap::Data),
    ItemFont,
    ItemTextField,
    ItemSprite,
}
