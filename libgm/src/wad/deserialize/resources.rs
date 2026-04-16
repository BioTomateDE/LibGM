use std::collections::HashMap;

use crate::prelude::*;
use crate::wad::deserialize::reader::DataReader;
use crate::wad::elements::texture_page_item::GMTexturePageItem;
use crate::wad::reference::GMRef;

impl DataReader<'_> {
    /// Read a standard GameMaker string reference.
    pub fn read_gm_string(&mut self) -> Result<String> {
        let occurrence_position = self.read_u32()?;
        self.read_gm_str(occurrence_position)
            .context("reading GameMaker String reference")
    }

    pub fn read_gm_string_opt(&mut self) -> Result<Option<String>> {
        let occurrence_position = self.read_u32()?;
        if occurrence_position == 0 {
            return Ok(None);
        }

        let string = self
            .read_gm_str(occurrence_position)
            .context("reading optional GameMaker String reference")?;

        Ok(Some(string))
    }

    fn read_gm_str(&mut self, occurrence_position: u32) -> Result<String> {
        let saved_pos = self.cur_pos;
        let saved_chunk = self.chunk.clone();

        self.cur_pos = occurrence_position
            .checked_sub(4)
            .ok_or_else(|| format!("Occurrence position {occurrence_position} is too low"))?;
        self.chunk = self.string_chunk.clone();

        let length = self.read_u32().context("reading GameMaker String length")?;

        let string = self
            .read_literal_string(length)
            .context("reading GameMaker String")?;

        self.cur_pos = saved_pos;
        self.chunk = saved_chunk;

        Ok(string)
    }

    pub fn read_gm_texture(&mut self) -> Result<GMRef<GMTexturePageItem>> {
        let occurrence_position = self.read_u32()?;
        self.resolve_tpag(occurrence_position)
    }

    pub fn read_gm_texture_opt(&mut self) -> Result<Option<GMRef<GMTexturePageItem>>> {
        let occurrence_position = self.read_u32()?;
        if occurrence_position == 0 {
            return Ok(None);
        }
        Ok(Some(self.resolve_tpag(occurrence_position)?))
    }

    fn resolve_tpag(&self, occurrence_position: u32) -> Result<GMRef<GMTexturePageItem>> {
        // You could probably make this a `remove()` (if that helps performance)?
        // At least UT/DR do not use any texture page items twice.
        let map: &HashMap<u32, GMRef<GMTexturePageItem>> = &self.texture_page_item_occurrences;
        if let Some(&tpag_ref) = map.get(&occurrence_position) {
            return Ok(tpag_ref);
        }
        Err(err!(
            "Could not read texture page item with occurrence position {} at pointer position {} \
             because it doesn't exist in the occurrence map with {} items",
            occurrence_position,
            self.cur_pos - 4,
            map.len(),
        ))
    }

    pub fn read_resource_by_id<T>(&mut self) -> Result<GMRef<T>> {
        const CTX: &str = "reading resource by ID";
        let number = self.read_u32().context(CTX)?;
        check_resource_limit(number).context(CTX)?;
        Ok(GMRef::new(number))
    }

    pub fn read_resource_by_id_opt<T>(&mut self) -> Result<Option<GMRef<T>>> {
        let number = self.read_i32()?;
        resource_opt_from_i32(number)
    }
}

pub fn resource_opt_from_i32<T>(number: i32) -> Result<Option<GMRef<T>>> {
    if number == -1 {
        return Ok(None);
    }
    let number = number as u32;
    check_resource_limit(number).context("parsing optional resource by id")?;
    Ok(Some(GMRef::new(number)))
}

fn check_resource_limit(number: u32) -> Result<()> {
    // Increase limit if not enough
    const FAILSAFE_COUNT: u32 = 500_000;
    if number < FAILSAFE_COUNT {
        return Ok(());
    }

    let signed = number as i32;
    let comment = if signed < 0 {
        format!(" (presumably {signed} as signed integer)")
    } else {
        String::new()
    };
    bail!("Resource ID {number}{comment} exceeds failsafe limit of {FAILSAFE_COUNT}");
}
