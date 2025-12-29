use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMElement, sound::GMSound},
        reference::GMRef,
        serialize::builder::DataBuilder,
    },
    prelude::*,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Audio {
    pub sound: GMRef<GMSound>,
    pub mode: i32,
}

impl GMElement for Audio {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let sound: GMRef<GMSound> = reader.read_resource_by_id()?;
        let mode = reader.read_i32()?;
        Ok(Self { sound, mode })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_resource_id(self.sound);
        builder.write_i32(self.mode);
        Ok(())
    }
}
