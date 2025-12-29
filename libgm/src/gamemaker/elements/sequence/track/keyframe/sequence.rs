use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMElement, sequence::GMSequence},
        reference::GMRef,
        serialize::builder::DataBuilder,
    },
    prelude::*,
};
#[derive(Debug, Clone, PartialEq)]
pub struct Sequence {
    pub sequence: GMRef<GMSequence>,
}

impl GMElement for Sequence {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let sequence: GMRef<GMSequence> = reader.read_resource_by_id()?;
        Ok(Self { sequence })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_resource_id(self.sequence);
        Ok(())
    }
}
