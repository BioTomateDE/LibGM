use crate::gamemaker::data::Endianness;
use crate::gamemaker::deserialize::reader::DataReader;
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::gm_version::GMVersion;
use crate::prelude::*;
use crate::util::assert::assert_int;
use crate::util::bench::Stopwatch;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GMChunk {
    pub start_pos: u32,
    pub end_pos: u32,
}

impl GMChunk {
    #[must_use]
    pub const fn length(&self) -> u32 {
        self.end_pos - self.start_pos
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.length() == 0
    }
}

impl DataReader<'_> {
    /// Read a `GameMaker` chunk name consisting of 4 ascii characters.
    /// Accounts for endianness; reversing the read chunk name in big endian mode.
    pub fn read_chunk_name(&mut self) -> Result<String> {
        let string: String = match self.read_literal_string(4) {
            Ok(str) => str,
            Err(_) if self.cur_pos == 4 => {
                bail!("Invalid data.win file; data doesn't start with 'FORM' string")
            }
            Err(e) => Err(e).context("parsing chunk name")?,
        };

        assert_int("Size of chunk name string", 4, string.len())?;
        if !string.bytes().all(|b| matches!(b, b'A'..=b'Z' | b'0'..=b'9')) {
            bail!("Chunk name {string:?} contains invalid characters")
        }

        // Chunk names are reversed in big endian
        if self.endianness == Endianness::Big {
            let mut bytes = string.into_bytes();
            bytes.reverse();
            // SAFETY: This operation is safe because the string was already checked to be ascii.
            return Ok(unsafe { String::from_utf8_unchecked(bytes) });
        }

        Ok(string)
    }

    pub fn read_chunk<T: GMChunkElement>(&mut self) -> Result<T> {
        let Some(chunk) = self.chunks.remove(T::NAME) else {
            return Ok(T::default());
        };

        let ctx = || format!("deserializing chunk '{}'", T::NAME);
        let stopwatch = Stopwatch::start();
        self.cur_pos = chunk.start_pos;
        self.chunk = chunk;

        let element = T::deserialize(self).with_context(ctx)?;

        // Last chunk does not get padding
        if T::NAME != self.last_chunk {
            self.read_chunk_padding().with_context(ctx)?;
        }

        integrity_assert! {
            self.cur_pos == self.chunk.end_pos,
            "Misaligned chunk '{}': expected chunk end position {} but reader is actually at position {} (diff: {})",
            T::NAME,
            self.chunk.end_pos,
            self.cur_pos,
            self.chunk.end_pos as i64 - self.cur_pos as i64,
        }

        log::trace!("Parsing chunk '{}' took {stopwatch}", T::NAME);
        Ok(element)
    }

    /// Potentially read padding at the end of the chunk, depending on the `GameMaker` version.
    fn read_chunk_padding(&mut self) -> Result<()> {
        // Padding only for GMS2+ and 1.9999+
        let ver: &GMVersion = &self.specified_version;
        let padding_elegible = ver.major >= 2 || (ver.major == 1 && ver.minor >= 9999);
        if !padding_elegible {
            return Ok(());
        }

        while !self.cur_pos.is_multiple_of(self.chunk_padding) {
            let byte: u8 = self.read_u8().context("reading chunk padding")?;
            if byte == 0 {
                continue;
            }

            // Byte is not zero => Padding is incorrect
            self.cur_pos -= 1; // Undo reading incorrect padding byte
            self.chunk_padding = if self.cur_pos % 4 == 0 { 4 } else { 1 };
            log::debug!("Set chunk padding to {}", self.chunk_padding);
            return Ok(());
        }

        // Padding was already set correctly
        Ok(())
    }

    /// Reads the specified `GameMaker` version in the GEN8 chunk.
    /// This only works if the GEN8 chunk still exists in the chunk map.
    ///
    /// This function should be called **after** parsing FORM but **before** reading any chunks.
    pub fn read_gen8_version(&mut self) -> Result<GMVersion> {
        const CTX: &str = "trying to read GEN8 `GameMaker` Version";
        let saved_pos = self.cur_pos;
        let saved_chunk: GMChunk = self.chunk.clone();
        self.chunk = self
            .chunks
            .get("GEN8")
            .cloned()
            .context("Chunk GEN8 does not exist")
            .context(CTX)?;
        self.cur_pos = self.chunk.start_pos + 44; // Skip to GEN8 `GameMaker` version
        let gm_version = GMVersion::deserialize(self).context(CTX)?;
        self.cur_pos = saved_pos;
        self.chunk = saved_chunk;
        Ok(gm_version)
    }
}
