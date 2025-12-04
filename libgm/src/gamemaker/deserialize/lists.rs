use crate::{
    gamemaker::{deserialize::reader::DataReader, elements::GMElement, reference::GMRef},
    prelude::*,
    util::{fmt::typename, init::vec_with_capacity},
};

impl DataReader<'_> {
    /// Reads a GameMaker simple list by calling the specified deserializer function for each element.
    ///
    /// Simple lists consist of a count followed by the elements' data in sequence.
    /// Includes a failsafe check to prevent excessive memory allocation from malformed data.
    fn read_simple_list_internal<T>(
        &mut self,
        count: u32,
        deserializer_fn: fn(&mut Self) -> Result<T>,
    ) -> Result<Vec<T>> {
        let mut elements: Vec<T> = vec_with_capacity(count).context("reading simple list")?;
        for _ in 0..count {
            let element: T = deserializer_fn(self).with_context(|| {
                format!(
                    "deserializing element {}/{} of {} simple list",
                    elements.len(),
                    count,
                    typename::<T>(),
                )
            })?;
            elements.push(element);
        }
        Ok(elements)
    }

    /// Reads a GameMaker simple list with a 32-bit count prefix.
    ///
    /// The list format is: `[count: u32][element_0][element_1]...[element_n]`
    pub fn read_simple_list<T: GMElement>(&mut self) -> Result<Vec<T>> {
        let count = self.read_u32()?;
        self.read_simple_list_internal(count, T::deserialize)
    }

    /// Reads a GameMaker simple list with a 16-bit count prefix.
    ///
    /// The list format is: `[count: u16][element_0][element_1]...[element_n]`
    /// Uses a smaller failsafe limit appropriate for short lists.
    pub fn read_simple_list_short<T: GMElement>(&mut self) -> Result<Vec<T>> {
        let count = self.read_u16()?;
        self.read_simple_list_internal(u32::from(count), T::deserialize)
    }

    /// Reads a simple list of resource IDs and wraps them in [`GMRef`].
    ///
    /// Each element is a 32-bit resource ID that gets resolved to a reference.
    pub fn read_simple_list_of_resource_ids<T>(&mut self) -> Result<Vec<GMRef<T>>> {
        let count = self.read_u32()?;
        self.read_simple_list_internal(count, Self::read_resource_by_id)
    }

    /// Reads a simple list of GameMaker string references.
    pub fn read_simple_list_of_strings(&mut self) -> Result<Vec<String>> {
        let count = self.read_u32()?;
        self.read_simple_list_internal(count, Self::read_gm_string)
    }

    pub fn read_pointer_list<T: GMElement>(&mut self) -> Result<Vec<T>> {
        // TODO implement 2024.11+ null pointers (unused asset removal)
        let pointers: Vec<u32> = self.read_simple_list()?;
        let count = pointers.len();

        let mut elements: Vec<T> = Vec::with_capacity(count);
        for (i, pointer) in pointers.into_iter().enumerate() {
            let element: T = self
                .read_pointer_element(pointer, i == count - 1)
                .with_context(|| {
                    format!(
                        "deserializing element {}/{} of {} pointer list",
                        i,
                        count,
                        typename::<T>(),
                    )
                })?;
            elements.push(element);
        }
        Ok(elements)
    }

    fn read_pointer_element<T: GMElement>(&mut self, pointer: u32, is_last: bool) -> Result<T> {
        T::deserialize_pre_padding(self)?;
        self.assert_pos(pointer, &typename::<T>())?;
        let element = T::deserialize(self)?;
        T::deserialize_post_padding(self, is_last)?;
        Ok(element)
    }

    /// Called `UndertaleAlignUpdatedListChunk` in UTMT.
    /// Used for BGND (and STRG).
    pub fn read_aligned_list_chunk<T: GMElement>(
        &mut self,
        alignment: u32,
        is_aligned: &mut bool,
    ) -> Result<Vec<T>> {
        let pointers: Vec<u32> = self.read_simple_list()?;
        let count = pointers.len();
        let mut elements: Vec<T> = Vec::with_capacity(count);
        *is_aligned = pointers.iter().all(|&p| p % alignment == 0);

        for pointer in pointers {
            if *is_aligned {
                self.align(alignment)?;
            }
            let element: T = self.read_pointer_element(pointer, false).with_context(|| {
                format!(
                    "deserializing element {}/{} of {}aligned {} pointer list",
                    elements.len(),
                    count,
                    if *is_aligned { "" } else { "un" },
                    typename::<T>(),
                )
            })?;
            elements.push(element);
        }
        Ok(elements)
    }
}
