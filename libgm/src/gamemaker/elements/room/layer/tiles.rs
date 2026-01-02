use std::cmp::min;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMElement, background::GMBackground},
        reference::GMRef,
        serialize::builder::DataBuilder,
    },
    prelude::*,
    util::init::vec_with_capacity,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tiles {
    pub background: Option<GMRef<GMBackground>>,
    /// Flattened 2D Array. Access using `tile_data[row + width * col]`.
    pub tile_data: Vec<u32>,
    pub width: u32,
    pub height: u32,
}

impl GMElement for Tiles {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let background: Option<GMRef<GMBackground>> = reader.read_resource_by_id_opt()?;
        let width = reader.read_u32()?;
        let height = reader.read_u32()?;
        let mut tile_data: Vec<u32> = vec_with_capacity(width * height)?;

        if reader.general_info.is_version_at_least((2024, 2)) {
            Self::read_compressed_tile_data(reader, &mut tile_data)?;
        } else {
            for _y in 0..height {
                for _x in 0..width {
                    tile_data.push(reader.read_u32()?);
                }
            }
        }

        Ok(Self { background, tile_data, width, height })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_resource_id_opt(self.background);
        builder.write_u32(self.width);
        builder.write_u32(self.height);
        if builder.is_version_at_least((2024, 2)) {
            self.build_compressed_tile_data(builder);
        } else {
            for id in &self.tile_data {
                builder.write_u32(*id);
            }
        }
        Ok(())
    }
}
impl Tiles {
    fn read_compressed_tile_data(reader: &mut DataReader, tile_data: &mut Vec<u32>) -> Result<()> {
        let total_size: usize = tile_data.capacity();
        if total_size == 0 {
            return Ok(());
        }

        'outer: loop {
            let length = reader.read_u8()?;
            if length >= 128 {
                // Repeat run
                let run_length: u8 = (length & 0x7F) + 1;
                let tile = reader.read_u32()?;
                for _ in 0..run_length {
                    tile_data.push(tile);
                    if tile_data.len() >= total_size {
                        break 'outer;
                    }
                }
            } else {
                // Verbatim run
                for _ in 0..length {
                    let tile = reader.read_u32()?;
                    tile_data.push(tile);
                    if tile_data.len() >= total_size {
                        break 'outer;
                    }
                }
            }
        }

        // Due to a GMAC bug, 2 blank tiles are inserted into the layer
        // If the last 2 tiles in the layer are different.
        // This is a certified YoyoGames moment right here.
        let has_padding: bool = if tile_data.len() == 1 {
            true // Single tile always has padding
        } else if tile_data.len() >= 2 {
            let len = tile_data.len();
            tile_data[len - 1] != tile_data[len - 2]
        } else {
            false // no tiles => no padding (should never happen though?)
        };
        if has_padding {
            let length = reader.read_u8()?;
            let tile = reader.read_u32()?;

            // Sanity check: run of 2 empty tiles
            if length != 0x81 {
                bail!(
                    "Expected 0x81 for run length of compressed tile data padding; got 0x{length:02X}"
                );
            }
            if tile as i32 != -1 {
                bail!("Expected -1 for tile of compressed tile data padding; got 0x{length:02X}");
            }
        }

        if reader.general_info.is_version_at_least((2024, 4)) {
            reader.align(4)?;
        }
        Ok(())
    }

    fn build_compressed_tile_data(&self, builder: &mut DataBuilder) {
        let tile_count: usize = self.tile_data.len();
        if tile_count == 0 {
            return;
        }

        // Perform run-length encoding using process identical to GameMaker's logic.
        // This only serializes data when outputting a repeat run, upon which the
        // Previous verbatim run is serialized first.
        // We also iterate in 1D, which requires some division and modulo to work with
        // The 2D array we have for representation here.
        let mut last_tile: u32 = self.tile_data[0];
        let mut num_verbatim: i32 = 0;
        let mut verbatim_start: i32 = 0;
        let mut i = 1;

        // Note: we go out of bounds to ensure a repeat run at the end
        while i <= tile_count + 1 {
            let mut curr_tile: u32 = if i >= tile_count {
                u32::MAX
            } else {
                self.tile_data[i]
            };
            i += 1;

            if curr_tile != last_tile {
                // We have different tiles, so just increase the number of tiles in this verbatim run
                num_verbatim += 1;
                last_tile = curr_tile;
                continue;
            }

            // We have two tiles in a row - construct a repeating run.
            // Figure out how far this repeat goes, first.
            let mut num_repeats: i32 = 2;
            while i < tile_count {
                if curr_tile != self.tile_data[i] {
                    break;
                }
                num_repeats += 1;
                i += 1;
            }

            // Serialize the preceding verbatim run, splitting into 127-length chunks
            while num_verbatim > 0 {
                let num_to_write: i32 = min(num_verbatim, 127);
                builder.write_u8(num_to_write as u8);
                for j in 0..num_to_write {
                    let tile: u32 = self.tile_data[(verbatim_start + j) as usize];
                    builder.write_u32(tile);
                }
                num_verbatim -= num_to_write;
                verbatim_start += num_to_write;
            }

            // Serialize this repeat run, splitting into 128-length chunks
            while num_repeats > 0 {
                let num_to_write: i32 = min(num_verbatim, 128);
                builder.write_u8((num_to_write as u8 - 1) | 0x80);
                builder.write_u32(last_tile);
                num_repeats -= num_to_write;
            }

            // Update our current tile to be the one after the run
            curr_tile = if i >= tile_count {
                0
            } else {
                self.tile_data[i]
            };

            // Update the start of our next verbatim run, and move on
            verbatim_start = i as i32;
            num_verbatim = 0;
            i += 1;
            last_tile = curr_tile;
        }

        if builder.is_version_at_least((2024, 4)) {
            builder.align(4);
        }
    }
}
