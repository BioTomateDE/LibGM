// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::GMElement;
use crate::wad::parse::reader::DataReader;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Moment {
    /// Should be 0 if none, 1 if there's a message
    /// (whatever that means)
    pub internal_count: i32,
    pub event: GMRef<String>,
}

impl GMElement for Moment {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let internal_count = reader.read_i32()?;
        let event: GMRef<String> = if internal_count > 0 {
            reader.read_gm_string()?
        } else {
            GMRef::none()
        };
        Ok(Self { internal_count, event })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(self.internal_count);
        if self.internal_count > 0 {
            builder.write_gm_string(self.event)?;
        }
        // FIXME: maybe there should be null written if event string not set?
        Ok(())
    }
}
