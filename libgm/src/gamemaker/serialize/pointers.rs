use std::collections::HashMap;

use crate::{
    gamemaker::serialize::builder::DataBuilder,
    prelude::*,
    util::{bench::Stopwatch, fmt::typename},
};

impl DataBuilder<'_> {
    /// Create a placeholder pointer at the current position in the chunk and remember
    /// its data position paired with the target `GameMaker` element's memory address.
    ///
    /// This will later be resolved by calling [`Self::resolve_pointer`]; replacing the
    /// pointer placeholder with the written data position of the target `GameMaker` element.
    /// ___
    /// This system exists because it is virtually impossible to
    /// predict which data position a `GameMaker` element will be written to.
    /// Circular references and writing order would make
    /// predicting these pointer resource positions even harder.
    /// ___
    /// This function should NOT be called for `GameMaker References by ID`;
    /// use their `DataBuilder::write_gm_x()` methods instead.
    pub fn write_pointer<T>(&mut self, element: &T) {
        let raw_pointer: *const T = std::ptr::from_ref(element);
        let memory_address = raw_pointer as usize;

        let placeholder_position: u32 = self.len() as u32;

        self.write_u32(0xDEAD_C0DE);
        self.pointer_placeholder_positions
            .push((placeholder_position, memory_address));
    }

    /// Optionally writes a pointer to the given [`Option`] value.
    /// - If [`Some`], writes a pointer to the contained value using [`Self::write_pointer`].
    /// - If [`None`], writes a null pointer (0) using [`Self::write_i32`].
    pub fn write_pointer_opt<T>(&mut self, element: &Option<T>) {
        if let Some(elem) = element {
            self.write_pointer(elem);
        } else {
            self.write_i32(0);
        }
    }

    /// Store the written `GameMaker` element's data position paired with its memory address in the pointer resource pool.
    /// The element's position corresponds to the data builder's current position,
    /// since this method should get called when the element is serialized.
    pub fn resolve_pointer<T>(&mut self, element: &T) -> Result<()> {
        let raw_pointer: *const T = std::ptr::from_ref(element);
        let memory_address = raw_pointer as usize;

        let resource_position: u32 = self.len() as u32;

        let old_resource_pos_opt = self
            .pointer_resource_positions
            .insert(memory_address, resource_position);

        let Some(old_resource_pos) = old_resource_pos_opt else {
            return Ok(());
        };

        bail!(
            "Pointer placeholder for {} with memory address {} already resolved \
            to data position {}; tried to resolve again to data position {}",
            typename::<T>(),
            memory_address,
            old_resource_pos,
            resource_position,
        );
    }

    /// Resolve pointer placeholders to their actual data positon they point to.
    /// This function should be called once after writing all chunks.
    pub fn connect_pointer_placeholders(&mut self) -> Result<()> {
        let stopwatch = Stopwatch::start();

        let placeholders: Vec<(u32, usize)> =
            std::mem::take(&mut self.pointer_placeholder_positions);
        let resources: HashMap<usize, u32> = std::mem::take(&mut self.pointer_resource_positions);

        let placeholder_count = placeholders.len();
        let resource_count = resources.len();

        for (placeholder_data_pos, element_mem_addr) in placeholders {
            let resource_data_pos: u32 = *resources.get(&element_mem_addr).ok_or_else(|| {
                format!(
                    "Could not resolve pointer placeholder with data position \
                {placeholder_data_pos} and memory address {element_mem_addr}"
                )
            })?;

            // Overwrite the `0xDEAD_C0DE` placeholder.
            // This `?` should never fail.
            self.overwrite_i32(resource_data_pos as i32, placeholder_data_pos as usize)?;
        }

        log::trace!(
            "Resolving {placeholder_count} pointer placeholders to \
            {resource_count} resources took {stopwatch}"
        );
        Ok(())
    }
}
