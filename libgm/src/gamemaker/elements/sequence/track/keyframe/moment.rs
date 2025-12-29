use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Moment {
    /// Should be 0 if none, 1 if there's a message
    /// (whatever that means)
    pub internal_count: i32,
    pub event: Option<String>,
}

impl GMElement for Moment {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let internal_count = reader.read_i32()?;
        let event: Option<String> = if internal_count > 0 {
            Some(reader.read_gm_string()?)
        } else {
            None
        };
        Ok(Self { internal_count, event })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(self.internal_count);
        if let Some(ref event) = self.event {
            builder.write_gm_string(event);
        }
        // FIXME: maybe there should be null written if event string not set?
        Ok(())
    }
}
