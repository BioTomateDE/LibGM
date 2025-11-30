use crate::{
    gamemaker::{
        data::Endianness,
        elements::GMChunkElement,
        serialize::builder::{DataBuilder, LastChunk},
    },
    prelude::*,
    util::bench::Stopwatch,
};

impl DataBuilder<'_> {
    /// Write a 4 character ASCII GameMaker chunk name.
    /// Accounts for endianness (chunk names in big endian are reversed).
    pub fn write_chunk_name(&mut self, name: &str) -> Result<()> {
        if name.len() != 4 {
            bail!(
                "Expected chunk name '{}' to be 4 characters long; but is actually {} characters long",
                name,
                name.chars().count(),
            );
        }

        let mut bytes: [u8; 4] = name.as_bytes().try_into().map_err(|_| {
            format!(
                "Expected chunk name '{}' to be 4 bytes long; but it's actually {} bytes long",
                name,
                name.len(),
            )
        })?;

        if self.gm_data.endianness == Endianness::Big {
            bytes.reverse();
        }

        self.write_bytes(&bytes);
        Ok(())
    }

    /// Writes a GameMaker data chunk.
    /// Skips the chunk if the element does not exist.
    ///
    /// Appends padding if required by the GameMaker version.
    /// This padding has to then be manually cut off for the last chunk in the data file.
    pub fn build_chunk<T: GMChunkElement>(&mut self, element: &T) -> Result<()> {
        let name: &str = T::NAME;
        if !element.exists() {
            return Ok(());
        }

        let stopwatch = Stopwatch::start();

        self.write_chunk_name(name)
            .expect("Constant chunk name is invalid");
        self.write_u32(0xDEAD_C0DE); // Chunk length placeholder
        let start_pos: usize = self.len();
        let length_pos = start_pos - 4;

        element
            .serialize(self)
            .with_context(|| format!("serializing chunk '{name}'"))?;

        // Write padding in these versions
        let padding_start_pos = self.len();
        let ver = &self.gm_data.general_info.version;
        if ver.major >= 2 || (ver.major == 1 && ver.build >= 9999) {
            self.align(self.gm_data.chunk_padding);
        }

        // Since the padding should not get written for the last chunk,
        // set the `last_chunk` field to potentially remove the padding later.
        self.last_chunk = LastChunk { length_pos, padding_start_pos };

        // Resolve chunk length placeholder
        let chunk_length: usize = self.len() - start_pos;
        self.overwrite_usize(chunk_length, length_pos)
            .expect("Chunk length overwrite position out of bounds");

        log::trace!("Building chunk '{name}' took {stopwatch}");
        Ok(())
    }

    /// Remove potential padding from the chunk written last
    /// since the the data file's last chunk does not get padding.
    pub fn remove_last_chunk_padding(&mut self) {
        let last = self.last_chunk.clone();
        let chunk_length = last.padding_start_pos - last.length_pos - 4;
        self.truncate_bytes(last.padding_start_pos);
        self.overwrite_usize(chunk_length, last.length_pos).unwrap();
    }
}
