use crate::gamemaker::deserialize::reader::DataReader;
use crate::gamemaker::elements::GMElement;
use crate::gamemaker::elements::texture_page_items::GMTexturePageItem;
use crate::gamemaker::reference::GMRef;
use crate::prelude::*;
use crate::util::fmt::typename;
use std::collections::HashMap;

impl DataReader<'_> {
    /// Read a standard `GameMaker` string reference.
    pub fn read_gm_string(&mut self) -> Result<String> {
        let occurrence_position = self.read_u32()?;
        self.read_gm_str(occurrence_position)
            .context("reading optional GameMaker String reference")
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

        self.cur_pos = occurrence_position - 4;
        self.chunk = self.string_chunk.clone();

        let length = self.read_u32().context("reading GameMaker String length")?;
        let string = self.read_literal_string(length).context("reading GameMaker String")?;

        self.cur_pos = saved_pos;
        self.chunk = saved_chunk;

        Ok(string)
    }

    pub fn read_gm_texture(&mut self) -> Result<GMRef<GMTexturePageItem>> {
        let occurrence_position = self.read_u32()?;
        self.resolve_occurrence(occurrence_position, &self.texture_page_item_occurrences)
    }

    pub fn read_gm_texture_opt(&mut self) -> Result<Option<GMRef<GMTexturePageItem>>> {
        let occurrence_position = self.read_u32()?;
        if occurrence_position == 0 {
            return Ok(None);
        }
        Ok(Some(self.resolve_occurrence(
            occurrence_position,
            &self.texture_page_item_occurrences,
        )?))
    }

    fn resolve_occurrence<T: GMElement>(
        &self,
        occurrence_position: u32,
        occurrence_map: &HashMap<u32, GMRef<T>>,
    ) -> Result<GMRef<T>> {
        match occurrence_map.get(&occurrence_position) {
            Some(gm_ref) => Ok(*gm_ref),
            None => bail!(
                "Could not read {} with occurrence position {} at pointer position {} \
                because it doesn't exist in the occurrence map with {} items",
                typename::<T>(),
                occurrence_position,
                self.cur_pos - 4,
                occurrence_map.len(),
            ),
        }
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
    const CTX: &str = "parsing optional resource by ID";
    if number == -1 {
        return Ok(None);
    }
    let number: u32 = number
        .try_into()
        .ok()
        .with_context(|| format!("Invalid negative number {number} (0x{number:08X})"))
        .context(CTX)?;
    check_resource_limit(number).context(CTX)?;
    Ok(Some(GMRef::new(number)))
}

fn check_resource_limit(number: u32) -> Result<()> {
    // Increase limit if not enough
    const FAILSAFE_COUNT: u32 = 500_000;
    integrity_assert! {
        number < FAILSAFE_COUNT,
        "Number {number} exceeds failsafe limit of {FAILSAFE_COUNT}"
    }
    Ok(())
}
