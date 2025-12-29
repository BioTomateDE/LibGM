//! # What are pointers
//! When building GameMaker data files, you frequently need to write "pointers".
//! "Pointer" in the context of GameMaker internals: a 32 bit integer that indicates
//! the absolute data position at which a resource is located.
//! For example, strings are (almost) always stored as pointers, allowing the runner
//! to do a simple addition to get to the UTF-8 string:
//! `data_start_mem_address + pointer = string_mem_address`.
//!
//! # Why placeholders are needed when building data
//! GameMaker chunks and elements are (mostly) written in an arbitrary order.
//! Any element in any chunk can refer to any other element in any chunk.
//! They can refer to elements previously declared in the file or to elements
//! that are "not yet" declared. They can even refer to themselves!
//! It's practically impossible to predict where an element (by ID)
//! will be written to.
//! Placeholders solve this issue:
//! Instead of writing the resource data position immediately,
//! you just write a placeholder/stub (`0xDEADC0DE`) for now.
//! The pointer placeholder is remembered in `DataBuilder.pointer_placeholder_positions`
//! with an associated **memory address**.
//! Then, when an element (that may be pointed to) is written,
//! you store the resolved data position of that element in `DataBuilder.pointer_resource_positions`.
//! (Note: this may happen before or after any placeholder to this element was written.)
//! At the very end, when everything has been built, you can match the pointer placeholders to
//! their resolved positions by comparing (hashing) element memory addresses.
//! The `0xDEADC0DE` bytes are then overwritten with the actual resource position.
//!
//!
//! # Why memory addresses
//! It's the simplest and most maintainable way
//! of keep track of which GameMaker element is which.
//!
//! You could use an enum instead that keeps track of element type and index.
//! I actually used this previously before I switched to memory addresses!
//! This enum sucks for multiple reasons though:
//! * Abstraction: when I rewrote this library with proper traits ([`GMElement`]),
//!   I lost context of GameMaker lists; the index is no longer known when (de)serializing a list element.
//! * Maintainability: Every pointer somewhere down the line needs an enum variant
//!   with N stacked indices. etc
//! * Performance: Using memory addresses is faster than these weird enums;
//!   it's literally just `lea ebx, [eax+FIELD_OFFSET]` in x86.
//!
//! # Drawbacks / Things to note
//! These memory addresses seem unstable, right?
//! And they are, if under the wrong conditions.
//! Using memory addresses as an identifier for GameMaker elements requires:
//! * The ID should always be the same for the same element instance,
//!   while the serialization is in progress.
//! * The ID should be unique; no other elements (that can be pointed to)
//!   should ever get the same ID.
//!
//! # Why this is still sound
//! The way I use memory addresses fulfils these requirements:
//! * Memory address stays the same, as long as the struct is not moved or reallocated.
//! * Nothing ever gets moved or reallocated, because [`GMData`] is borrowed immutably:
//!   `&gm_data`. This disables moving (taking ownership) and prevents reallocation,
//!   for example because of vector pushes.
//! * Memory addresses are unique for each struct field, as long as their size is
//!   greater than zero (ZSTs are not used anywhere in [`GMData`]).
//!
//! One key part is missing here:
//! You could use both a struct and a field of struct as a pointer placeholder.
//! If you use both, their addresses could be the same:
//! `struct_mem_address + field_offset = field_mem_address` could backfire if `offset` is zero.
//! You need to be careful here: Rust does not gurantee struct layout by default
//! and may reorder fields to optimise space.
//!
//! If you encounter a struct that is used as a pointer placeholder and one of its
//! fields is too:
//! 1. Annotate the struct with `#[repr(C)]`
//! 2. Make sure that the pointer placeholder field is not the first one in definition
//!
//! This prevents address collisions.

use std::collections::HashMap;

use crate::{
    gamemaker::serialize::builder::DataBuilder,
    prelude::*,
    util::{bench::Stopwatch, fmt::typename},
};

impl DataBuilder<'_> {
    /// Create a placeholder pointer at the current position in the chunk and remember
    /// its data position paired with the target GameMaker element's memory address.
    ///
    /// This will later be resolved by calling [`Self::resolve_pointer`]; replacing the
    /// pointer placeholder with the written data position of the target GameMaker element.
    /// ___
    /// This system exists because it is virtually impossible to
    /// predict which data position a GameMaker element will be written to.
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

    /// Store the written GameMaker element's data position paired with its memory address in the pointer resource pool.
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
