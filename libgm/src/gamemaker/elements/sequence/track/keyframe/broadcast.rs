use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
    util::init::vec_with_capacity,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BroadcastMessage {
    pub messages: Vec<String>,
}

impl GMElement for BroadcastMessage {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let count = reader.read_u32()?;
        let mut messages: Vec<String> = vec_with_capacity(count)?;
        for _ in 0..count {
            messages.push(reader.read_gm_string()?);
        }
        Ok(Self { messages })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_usize(self.messages.len())?;
        for message in &self.messages {
            builder.write_gm_string(message);
        }
        Ok(())
    }
}
