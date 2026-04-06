use std::fmt::Display;
use std::fmt::UpperHex;

use crate::gml::instruction::DataType;
use crate::prelude::*;
use crate::wad::deserialize::reader::DataReader;

impl DataReader<'_> {
    /// Ensures the reader is at the specified position.
    /// This only happens if `options.verify_alignment` is true.
    #[track_caller]
    pub fn assert_pos(&self, position: u32, pointer_name: &'static str) -> Result<()> {
        if cfg!(not(feature = "check-integrity")) {
            return Ok(());
        }

        if self.cur_pos == position {
            return Ok(());
        }

        self.warn_invalid_align(format!(
            "{} pointer is misaligned: expected position {} but reader is actually at {} (diff: \
             {})",
            pointer_name,
            position,
            self.cur_pos,
            i64::from(position) - i64::from(self.cur_pos),
        ))
    }

    pub fn read_gms2_chunk_version(&mut self, desc: &'static str) -> Result<()> {
        let chunk_version = self.read_u32()?;
        self.assert_int(chunk_version, 1, desc)?;
        Ok(())
    }

    /// Returns an error if `reader.options.verify_constants` is
    /// enabled, otherwise only prints a warning log.
    #[track_caller]
    pub fn warn_invalid_const(&self, message: String) -> Result<()> {
        if self.options.verify_constants {
            Err(Error::new(message))
        } else {
            log::warn!("{message}");
            Ok(())
        }
    }

    /// Returns an error if `reader.options.verify_alignment` is
    /// enabled, otherwise only prints a warning log.
    #[track_caller]
    pub fn warn_invalid_align(&self, message: String) -> Result<()> {
        if self.options.verify_alignment {
            Err(Error::new(message))
        } else {
            log::warn!("{message}");
            Ok(())
        }
    }

    #[track_caller]
    pub fn assert_int<I: Copy + Eq + Display + UpperHex>(
        &self,
        actual: I,
        expected: I,
        description: &'static str,
    ) -> Result<()> {
        if cfg!(not(feature = "check-integrity")) {
            return Ok(());
        }

        if expected == actual {
            return Ok(());
        }

        let width = size_of::<I>() * 2;
        self.warn_invalid_const(format!(
            "Expected {description} to be {expected} but it is actually {actual} \
             (0x{actual:0width$X})",
        ))
    }

    #[track_caller]
    pub fn assert_bool(
        &self,
        actual: bool,
        expected: bool,
        description: &'static str,
    ) -> Result<()> {
        if cfg!(not(feature = "check-integrity")) {
            return Ok(());
        }

        if expected == actual {
            return Ok(());
        }

        self.warn_invalid_const(format!(
            "Expected {description} to be {expected} but it is actually {actual}",
        ))
    }

    #[track_caller]
    pub fn assert_data_type(
        &self,
        actual: DataType,
        expected: DataType,
        description: &'static str,
    ) -> Result<()> {
        if cfg!(not(feature = "check-integrity")) {
            return Ok(());
        }

        if expected == actual {
            return Ok(());
        }

        self.warn_invalid_const(format!(
            "Expected {description} Data Type to be {expected:?} but it is actually {actual:?}"
        ))
    }
}
