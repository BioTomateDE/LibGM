use crate::{
    gamemaker::{
        chunk::ChunkName,
        data::Endianness,
        deserialize::reader::DataReader,
        elements::{GMChunk, GMElement},
        gm_version::GMVersion,
    },
    prelude::*,
    util::{bench::Stopwatch, smallmap::SmallMap},
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChunkBounds {
    pub start_pos: u32,
    pub end_pos: u32,
}

impl ChunkBounds {
    #[must_use]
    pub const fn length(&self) -> u32 {
        self.end_pos - self.start_pos
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.length() == 0
    }
}

/// The number of all known GameMaker chunks (excluding debug chunks).
const KNOWN_CHUNK_COUNT: usize = 35;

#[derive(Debug, Default)]
pub struct Chunks(SmallMap<ChunkName, ChunkBounds>);

impl Chunks {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(SmallMap::with_capacity(KNOWN_CHUNK_COUNT))
    }

    #[inline]
    #[must_use]
    pub const fn count(&self) -> usize {
        self.0.len()
    }

    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn push(&mut self, name: ChunkName, chunk: ChunkBounds) -> Result<()> {
        if self.0.contains_key(&name) {
            bail!("Chunk {name:?} is defined multiple times");
        }
        self.0.insert(name, chunk);
        Ok(())
    }

    pub fn contains(&self, name: &'static str) -> bool {
        self.0.contains_key(&ChunkName::new(name))
    }

    #[inline]
    #[must_use]
    pub fn get(&self, name: &'static str) -> Option<ChunkBounds> {
        self.get_by_chunkname(ChunkName::new(name))
    }

    #[inline]
    #[must_use]
    pub fn get_by_chunkname(&self, name: ChunkName) -> Option<ChunkBounds> {
        self.0.get(&name).cloned()
    }

    #[inline]
    #[must_use]
    pub fn remove(&mut self, name: ChunkName) -> Option<ChunkBounds> {
        self.0.remove(&name)
    }

    #[inline]
    pub fn chunk_names(&self) -> impl Iterator<Item = ChunkName> {
        self.0.keys().copied()
    }
}

impl DataReader<'_> {
    /// Read a GameMaker chunk name consisting of 4 ascii characters.
    /// Accounts for endianness; reversing the read chunk name in big endian mode.
    pub fn read_chunk_name(&mut self) -> Result<ChunkName> {
        let mut bytes: [u8; 4] = self.read_bytes_const().cloned()?;

        if self.endianness == Endianness::Big {
            bytes.reverse();
        }

        let chunk_name = ChunkName::from_bytes(bytes)?;
        Ok(chunk_name)
    }

    pub fn read_chunk<T: GMChunk>(&mut self) -> Result<T> {
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

        if self.cur_pos != self.chunk.end_pos {
            bail!(
                "Misaligned chunk '{}': expected chunk end position {} \
                but the reader is actually at position {} (diff: {})",
                T::NAME,
                self.chunk.end_pos,
                self.cur_pos,
                i64::from(self.chunk.end_pos) - i64::from(self.cur_pos),
            );
        }

        log::trace!("Parsing chunk '{}' took {stopwatch}", T::NAME);
        Ok(element)
    }

    /// Potentially read padding at the end of the chunk, depending on the GameMaker version.
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
            self.chunk_padding = if self.cur_pos.is_multiple_of(4) { 4 } else { 1 };
            log::debug!("Set chunk padding to {}", self.chunk_padding);
            return Ok(());
        }

        // Padding was already set correctly
        Ok(())
    }

    /// Reads the specified GameMaker version in the GEN8 chunk.
    /// This only works if the GEN8 chunk still exists in the chunk map.
    ///
    /// This function should be called **after** parsing FORM but **before** reading any chunks.
    pub fn read_gen8_version(&mut self) -> Result<GMVersion> {
        const CTX: &str = "trying to read GEN8 GameMaker Version";
        let saved_pos = self.cur_pos;
        let saved_chunk: ChunkBounds = self.chunk.clone();
        self.chunk = self
            .chunks
            .get("GEN8")
            .ok_or("Chunk GEN8 does not exist")
            .context(CTX)?;
        self.cur_pos = self.chunk.start_pos + 44; // Skip to GEN8 GameMaker version
        let gm_version = GMVersion::deserialize(self).context(CTX)?;
        self.cur_pos = saved_pos;
        self.chunk = saved_chunk;
        Ok(gm_version)
    }
}
