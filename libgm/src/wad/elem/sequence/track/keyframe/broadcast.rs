// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::util::init::vec_with_capacity;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::GMElement;
use crate::wad::parse::reader::DataReader;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BroadcastMessage {
    pub messages: Vec<GMRef<String>>,
}

impl GMElement for BroadcastMessage {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let count = reader.read_u32()?;
        let mut messages: Vec<GMRef<String>> = vec_with_capacity(count)?;
        for _ in 0..count {
            messages.push(reader.read_gm_string()?);
        }
        Ok(Self { messages })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_usize(self.messages.len())?;
        for &message in &self.messages {
            builder.write_gm_string(message)?;
        }
        Ok(())
    }
}
