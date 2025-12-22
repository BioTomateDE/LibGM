use std::fmt::{Display, UpperHex};

use crate::{gamemaker::deserialize::reader::DataReader, gml::instruction::DataType, prelude::*};

impl DataReader<'_> {
    /// Ensures the reader is at the specified position.
    /// This only happens if `options.verify_alignment` is true.
    pub fn assert_pos(&self, position: u32, pointer_name: &'static str) -> Result<()> {
        if self.cur_pos == position {
            return Ok(());
        }

        let msg = format!(
            "{} pointer is misaligned: expected position {} but \
            reader is actually at {} (diff: {})",
            pointer_name,
            position,
            self.cur_pos,
            i64::from(position) - i64::from(self.cur_pos),
        );

        if self.options.verify_alignment {
            Err(Error::from(msg))
        } else {
            log::warn!("{msg}");
            Ok(())
        }
    }

    pub fn read_gms2_chunk_version(&mut self, desc: &'static str) -> Result<()> {
        let chunk_version = self.read_u32()?;
        self.assert_int(chunk_version, 1, desc)?;
        Ok(())
    }

    pub fn assert_int<I: Copy + Eq + Display + UpperHex>(
        &self,
        actual: I,
        expected: I,
        description: &'static str,
    ) -> Result<()> {
        if expected == actual {
            return Ok(());
        }

        let width = size_of::<I>() * 2;
        let msg = format!(
            "Expected {} to be {} but it is actually {} (0x{:0width$X})",
            description, expected, actual, actual,
        );
        self.handle_invalid_constant(msg)
    }

    pub fn assert_bool(
        &self,
        actual: bool,
        expected: bool,
        description: &'static str,
    ) -> Result<()> {
        if expected == actual {
            return Ok(());
        }

        let msg = format!(
            "Expected {} to be {} but it is actually {}",
            description, expected, actual,
        );
        self.handle_invalid_constant(msg)
    }

    pub fn assert_data_type(
        &self,
        actual: DataType,
        expected: DataType,
        description: &'static str,
    ) -> Result<()> {
        if expected == actual {
            return Ok(());
        }

        let msg = format!(
            "Expected {} Data Type to be {:?} but it is actually {:?}",
            description, expected, actual,
        );
        self.handle_invalid_constant(msg)
    }

    fn handle_invalid_constant(&self, message: String) -> Result<()> {
        if self.options.verify_constants {
            Err(Error::from(message))
        } else {
            log::warn!("{message}");
            Ok(())
        }
    }
}
