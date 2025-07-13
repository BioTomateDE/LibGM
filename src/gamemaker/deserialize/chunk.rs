use crate::gamemaker::deserialize::reader::DataReader;
use crate::gamemaker::element::{GMChunkElement, GMElement};
use crate::gamemaker::gm_version::GMVersion;
use crate::utility::Stopwatch;


#[derive(Debug, Clone)]
pub struct GMChunk {
    pub name: String,
    pub start_pos: usize,
    pub end_pos: usize,
    pub is_last_chunk: bool,
}


impl DataReader<'_> {
    /// Read chunk name (4 ascii characters)
    pub fn read_chunk_name(&mut self) -> Result<String, String> {
        if self.chunk.name != "FORM" {
            return Err(format!(
                "Reading a chunk name is only allowed in root; not in a chunk!
                Current chunk is called '{}' and has start position {} and end position {}",
                self.chunk.name, self.chunk.start_pos, self.chunk.end_pos,
            ))
        }
        let string: String = self.read_literal_string(4)
            .map_err(|e| if self.cur_pos == 4 {
                "Invalid data.win file; data doesn't start with 'FORM' string".to_string()
            } else {
                format!("Could not parse chunk name at position {}: {e}", self.cur_pos)
            })?;
        if string.len() != 4 {
            return Err(format!("Chunk name string \"{string}\" has length {} (chunk names need to be 4 chars long)", string.len()))
        }
        if !string.is_ascii() {
            return Err(format!("Chunk name string \"{string}\" is not ascii"))
        }
        Ok(string)
    }
    
    fn read_chunk_internal<T: GMChunkElement+GMElement>(&mut self, chunk: GMChunk) -> Result<T, String> {
        let stopwatch = Stopwatch::start();
        self.cur_pos = chunk.start_pos;
        self.chunk = chunk;

        let element = T::deserialize(self)?;
        self.read_chunk_padding()?;

        if self.cur_pos != self.chunk.end_pos {
            return Err(format!(
                "Misaligned chunk '{}': expected chunk end position {} but reader is actually at position {} (diff: {})",
                self.chunk.name, self.chunk.end_pos, self.cur_pos, self.chunk.end_pos as i64 - self.cur_pos as i64,
            ))
        }

        log::trace!("Parsing chunk '{}' took {stopwatch}", self.chunk.name);
        Ok(element)
    }

    fn read_chunk_padding(&mut self) -> Result<(), String> {
        if self.chunk.is_last_chunk {
            return Ok(())   // last chunk does not get padding
        }
        let ver: GMVersion = if self.general_info.exists {
            self.general_info.version.clone()
        } else {
            self.unstable_get_gm_version()?     // only happens before chunk GEN8 is read (STRG)
        };
        if !(ver.major >= 2 || (ver.major == 1 && ver.minor >= 9999)) {
            return Ok(())     // no padding before these versions
        }

        while self.cur_pos % self.chunk_padding != 0 {
            let byte: u8 = self.read_u8().map_err(|e| format!("{e}\n↳ while reading chunk padding"))?;
            if byte == 0 { continue }
            // byte is not zero => padding is incorrect
            self.cur_pos -= 1;  // undo reading incorrect padding byte
            self.chunk_padding = if self.cur_pos % 4 == 0 { 4 } else { 1 };
            log::debug!("Set chunk padding to {}", self.chunk_padding);
            return Ok(())
        }
        Ok(())    // padding was already set correctly
    }

    pub fn read_chunk_required<T: GMChunkElement + GMElement>(&mut self, chunk_name: &str) -> Result<T, String> {
        let chunk: GMChunk = self.chunks.get(chunk_name).ok_or_else(|| format!(
            "Required chunk '{}' not found in chunk hashmap with length {}",
            chunk_name, self.chunks.len(),
        ))?.clone();

        let element: T = self.read_chunk_internal(chunk)
            .map_err(|e| format!("{e}\n↳ while deserializing required chunk '{chunk_name}'"))?;

        // Remove the chunk only after chunk parsing completes.
        // Removing it earlier (e.g. when reading GEN8) would prevent
        // the padding handling from finding the GEN8 chunk in the map,
        // since the real GEN8 info is only set after this function returns.
        self.chunks.remove(chunk_name);
        Ok(element)
    }

    pub fn read_chunk_optional<T: GMChunkElement + GMElement>(&mut self, chunk_name: &str) -> Result<T, String> {
        let Some(chunk) = self.chunks.remove(chunk_name) else {
            log::trace!("Skipped parsing optional chunk '{chunk_name}'");
            return Ok(T::empty())
        };
        let element: T = self.read_chunk_internal(chunk)
            .map_err(|e| format!("{e}\n↳ while deserializing optional chunk '{chunk_name}'"))?;
        Ok(element)
    }

    fn unstable_get_gm_version(&mut self) -> Result<GMVersion, String> {
        let saved_pos: usize = self.cur_pos;
        let saved_chunk: GMChunk = self.chunk.clone();
        self.chunk = self.chunks.get("GEN8").cloned().ok_or("Chunk GEN8 does not exist while trying to (unstable) read gm version")?;
        self.cur_pos = self.chunk.start_pos + 44;   // skip to GEN8 GameMaker version
        let gm_version = GMVersion::deserialize(self)?;
        self.cur_pos = saved_pos;
        self.chunk = saved_chunk;
        Ok(gm_version)
    }

    pub fn assert_chunk_name(&self, chunk_name: &str) -> Result<(), String> {
        if self.chunk.name == chunk_name {
            Ok(())
        } else {
            Err(format!(
                "Expected chunk with name '{}'; got chunk with name '{}' (length: {})",
                self.chunk.name, chunk_name, chunk_name.len(),
            ))
        }
    }
}

