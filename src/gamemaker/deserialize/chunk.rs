use crate::gamemaker::data::Endianness;
use crate::gamemaker::deserialize::reader::DataReader;
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::gm_version::GMVersion;
use crate::integrity_assert;
use crate::prelude::*;
use crate::util::bench::Stopwatch;

#[derive(Debug, Clone)]
pub struct GMChunk {
    pub name: String,
    pub start_pos: usize,
    pub end_pos: usize,
    pub is_last_chunk: bool,
}

impl DataReader<'_> {
    /// Read a GameMaker chunk name consisting of 4 ascii characters.
    /// Accounts for endianness; reversing the read chunk name in big endian mode.
    pub fn read_chunk_name(&mut self) -> Result<String> {
        integrity_assert! {
            self.chunk.name == "FORM",
            "Reading a chunk name is only allowed in FORM; not in a chunk!
            Current chunk is called '{}' and has start position {} and end position {}",
            self.chunk.name, self.chunk.start_pos, self.chunk.end_pos,
        }

        let string: String = self.read_literal_string(4).map_err(|e| {
            if self.cur_pos == 4 {
                "Invalid data.win file; data doesn't start with 'FORM' string".to_string()
            } else {
                format!("Could not parse chunk name at position {}: {e}", self.cur_pos)
            }
        })?;

        if string.len() != 4 {
            bail!("Chunk name string {string:?} has size {}", string.len());
        }

        integrity_assert! {
            string.bytes().all(|b| matches!(b, b'A'..=b'Z' | b'0'..=b'9')),
            "Chunk name {string:?} contains invalid characters"
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

    fn read_chunk_internal<T: GMChunkElement + GMElement>(&mut self, chunk: GMChunk) -> Result<T> {
        let stopwatch = Stopwatch::start();
        self.cur_pos = chunk.start_pos;
        self.chunk = chunk;

        let element = T::deserialize(self)?;
        self.read_chunk_padding()?;

        integrity_assert! {
            self.cur_pos == self.chunk.end_pos,
            "Misaligned chunk '{}': expected chunk end position {} but reader is actually at position {} (diff: {})",
            self.chunk.name, self.chunk.end_pos, self.cur_pos, self.chunk.end_pos as i64 - self.cur_pos as i64,
        }

        log::trace!("Parsing chunk '{}' took {stopwatch}", self.chunk.name);
        Ok(element)
    }

    /// Potentially read padding at the end of the chunk, depending on the GameMaker version.
    fn read_chunk_padding(&mut self) -> Result<()> {
        if self.chunk.is_last_chunk {
            return Ok(()); // Last chunk does not get padding
        }
        let ver: GMVersion = if self.general_info.exists {
            self.general_info.version.clone()
        } else {
            self.unstable_get_gm_version()? // only happens before chunk GEN8 is read (STRG)
        };
        if !(ver.major >= 2 || (ver.major == 1 && ver.minor >= 9999)) {
            return Ok(()); // No padding before these versions
        }

        while self.cur_pos % self.chunk_padding != 0 {
            let byte: u8 = self.read_u8().context("reading chunk padding")?;
            if byte == 0 {
                continue;
            }
            // Byte is not zero => padding is incorrect
            self.cur_pos -= 1; // Undo reading incorrect padding byte
            self.chunk_padding = if self.cur_pos % 4 == 0 { 4 } else { 1 };
            log::debug!("Set chunk padding to {}", self.chunk_padding);
            return Ok(());
        }
        Ok(()) // Padding was already set correctly
    }

    pub fn read_chunk_required<T: GMChunkElement + GMElement>(&mut self, chunk_name: &str) -> Result<T> {
        let chunk: GMChunk = self.chunks.get(chunk_name).cloned().ok_or_else(|| {
            format!(
                "Required chunk '{}' not found in chunk hashmap with length {}",
                chunk_name,
                self.chunks.len(),
            )
        })?;

        let element: T = self
            .read_chunk_internal(chunk)
            .with_context(|| format!("deserializing required chunk '{chunk_name}'"))?;

        // Remove the chunk only after chunk parsing completes.
        // Removing it earlier (e.g. when reading GEN8) would prevent
        // the padding handling from finding the GEN8 chunk in the map,
        // Since the real GEN8 info is only set after this function returns.
        self.chunks.remove(chunk_name);
        Ok(element)
    }

    pub fn read_chunk_optional<T: GMChunkElement + GMElement>(&mut self, chunk_name: &str) -> Result<T> {
        let Some(chunk) = self.chunks.remove(chunk_name) else {
            log::trace!("Skipped parsing optional chunk '{chunk_name}'");
            return Ok(T::stub());
        };
        let element: T = self
            .read_chunk_internal(chunk)
            .with_context(|| format!("deserializing optional chunk '{chunk_name}'"))?;
        Ok(element)
    }

    fn unstable_get_gm_version(&mut self) -> Result<GMVersion> {
        const CTX: &str = "trying to unstable read GameMaker Version";
        let saved_pos: usize = self.cur_pos;
        let saved_chunk: GMChunk = self.chunk.clone();
        self.chunk = self
            .chunks
            .get("GEN8")
            .cloned()
            .context("Chunk GEN8 does not exist")
            .context(CTX)?;
        self.cur_pos = self.chunk.start_pos + 44; // Skip to GEN8 GameMaker version
        let gm_version = GMVersion::deserialize(self).context(CTX)?;
        self.cur_pos = saved_pos;
        self.chunk = saved_chunk;
        Ok(gm_version)
    }
}
