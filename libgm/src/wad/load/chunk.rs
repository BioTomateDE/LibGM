// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::util::bench::Stopwatch;
use crate::wad::chunk::ChunkName;
use crate::wad::load::reader::DataReader;
use crate::wad::version::GMVersion;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ChunkBounds {
    pub start_pos: u32,
    pub end_pos: u32,
}

impl ChunkBounds {
    #[must_use]
    pub const fn length(self) -> u32 {
        self.end_pos - self.start_pos
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.length() == 0
    }
}

/// The number of all known GameMaker chunks (excluding debug chunks).
const KNOWN_CHUNK_COUNT: usize = 35;

#[derive(Debug, Clone, Default)]
pub struct ChunkMap(Vec<(ChunkName, ChunkBounds)>);

impl ChunkMap {
    #[must_use]
    pub fn new() -> Self {
        Self(Vec::with_capacity(KNOWN_CHUNK_COUNT))
    }

    #[must_use]
    pub fn contains(&self, chunk_name: ChunkName) -> bool {
        for (name, _) in &self.0 {
            if *name == chunk_name {
                return true;
            }
        }
        false
    }

    pub fn push(&mut self, name: ChunkName, chunk: ChunkBounds) -> Result<()> {
        if self.contains(name) {
            bail!("Chunk {name:?} is defined multiple times");
        }
        self.0.push((name, chunk));
        Ok(())
    }

    #[must_use]
    pub fn get(&self, chunk_name: ChunkName) -> Option<ChunkBounds> {
        for &(name, bounds) in &self.0 {
            if name == chunk_name {
                return Some(bounds);
            }
        }
        None
    }
}

impl DataReader<'_> {
    pub fn read_chunk<T: GMChunk>(&mut self) -> Result<T> {
        // If the chunk doesn't exist, return the `default` stub.
        // This will also set `exists` to false.
        let Some(chunk) = self.chunks.get(T::NAME) else {
            return Ok(T::default());
        };

        let ctx = || format!("deserializing chunk {}", T::NAME);
        let stopwatch = Stopwatch::start();
        self.cur_pos = chunk.start_pos;
        self.chunk = chunk;

        let element = T::deserialize(self).ctx(ctx)?;

        // Last chunk does not get padding
        if T::NAME != self.last_chunk {
            self.read_chunk_padding();
        }

        if self.cur_pos != self.chunk.end_pos {
            self.handle_misaligned_chunk(T::NAME)?;
        }

        log::trace!("Parsing chunk {} took {stopwatch}", T::NAME);
        Ok(element)
    }

    fn handle_misaligned_chunk(&mut self, chunk_name: ChunkName) -> Result<()> {
        let pos = self.cur_pos;
        let end = self.chunk.end_pos;
        let start = self.chunk.start_pos;

        let missing = self.read_bytes_dyn(end - pos)?;
        let all_zero = missing.iter().all(|&byte| byte == 0);

        self.handle_invalid_align(format!(
            "Misaligned chunk {}: File specified chunk length {}, but only {} bytes were read \
             (difference: {}{})",
            chunk_name,
            self.chunk.length(),
            pos - start,
            end - pos,
            if all_zero {
                "; all remaining bytes are zero"
            } else {
                ""
            }
        ))?;
        Ok(())
    }

    /// Potentially read padding at the end of the chunk, depending on the format version.
    fn read_chunk_padding(&mut self) {
        // Padding only for GMS2+ and 1.0.0.9999+
        if self.version < GMVersion::Wad16Pad {
            return;
        }

        while !self.cur_pos.is_multiple_of(self.chunk_padding) {
            if self.read_u8().is_ok_and(|b| b == 0) {
                continue;
            }

            // Byte is not zero (or tried to read beyond chunk bounds) => Padding is incorrect
            self.chunk_padding = if self.cur_pos.is_multiple_of(4) { 4 } else { 1 };
            log::debug!("Set chunk padding to {}", self.chunk_padding);
            return;
        }

        // Padding was already set correctly
    }
}
