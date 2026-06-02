// SPDX-License-Identifier: GPL-3.0-only
use std::collections::HashMap;
use std::hint::cold_path;

use crate::prelude::*;
use crate::wad::elem::texture_page_item::GMTexturePageItem;
use crate::wad::parse::reader::DataReader;
use crate::wad::reference::GMRef;

impl DataReader<'_> {
    /// Read a standard GameMaker string reference.
    pub fn read_gm_string(&mut self) -> Result<GMRef<String>> {
        let occurrence_position = self.read_u32()?;
        if occurrence_position == 0 {
            return Ok(GMRef::none());
        }

        let map: &HashMap<u32, GMRef<String>> = &self.string_occurrences;
        if let Some(&strg_ref) = map.get(&occurrence_position) {
            return Ok(strg_ref);
        }

        cold_path();
        Err(err!(
            "Could not read string with occurrence position {} at pointer position {} because it \
             doesn't exist in the occurrence map with {} items",
            occurrence_position,
            self.cur_pos - 4,
            map.len(),
        ))
    }

    pub fn read_gm_texture(&mut self) -> Result<GMRef<GMTexturePageItem>> {
        let occurrence_position = self.read_u32()?;
        if occurrence_position == 0 {
            return Ok(GMRef::none());
        }

        let map: &HashMap<u32, GMRef<GMTexturePageItem>> = &self.texture_page_item_occurrences;
        if let Some(&tpag_ref) = map.get(&occurrence_position) {
            return Ok(tpag_ref);
        }

        cold_path();
        Err(err!(
            "Could not read texture page item with occurrence position {} at pointer position {} \
             because it doesn't exist in the occurrence map with {} items",
            occurrence_position,
            self.cur_pos - 4,
            map.len(),
        ))
    }

    pub fn read_resource_by_id<T>(&mut self) -> Result<GMRef<T>> {
        // const CTX: &str = "reading resource by ID";
        // let number = self.read_u32().context(CTX)?;
        // check_resource_limit(number).context(CTX)?;
        // Ok(GMRef::new(number))
        let number = self.read_i32()?;

        if number == -1 {
            return Ok(GMRef::none());
        }
        check_resource_limit(number as u32).context("parsing resource by id")?;
        Ok(GMRef::new(number))
    }
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
