use crate::{
    gamemaker::{deserialize::reader::DataReader, elements::GMElement},
    prelude::*,
    util::{fmt::typename, init::vec_with_capacity},
};

impl DataReader<'_> {
    /// Reads a GameMaker simple list with a 32-bit count prefix.
    ///
    /// Simple lists consist of a count followed by the elements' data in sequence.
    /// Includes a failsafe check to prevent excessive memory allocation from malformed data.
    ///
    /// The list format is: `[count: u32][element_0][element_1]...[element_n]`
    pub fn read_simple_list<T: GMElement>(&mut self) -> Result<Vec<T>> {
        let count = self.read_u32()?;
        let mut elements: Vec<T> = vec_with_capacity(count).context("reading simple list")?;

        for _ in 0..count {
            let element = T::deserialize(self).with_context(|| {
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

    /// Reads a GameMaker simple list with a 16-bit count prefix.
    ///
    /// Simple lists consist of a count followed by the elements' data in sequence.
    /// Includes a failsafe check to prevent excessive memory allocation from malformed data.
    ///
    /// The list format is: `[count: u16][element_0][element_1]...[element_n]`
    pub fn read_simple_list_short<T: GMElement>(&mut self) -> Result<Vec<T>> {
        let count = u32::from(self.read_u16()?);
        let mut elements: Vec<T> = vec_with_capacity(count).context("reading short simple list")?;

        for _ in 0..count {
            let element = T::deserialize(self).with_context(|| {
                format!(
                    "deserializing element {}/{} of {} short simple list",
                    elements.len(),
                    count,
                    typename::<T>(),
                )
            })?;
            elements.push(element);
        }

        Ok(elements)
    }

    pub fn read_pointer_list<T: GMElement>(&mut self) -> Result<Vec<T>> {
        // TODO(important): implement 2024.11+ null pointers (unused asset removal)
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

        // Manually assert position
        if self.cur_pos != pointer {
            let name = typename::<T>();

            if pointer == 0 {
                bail!(
                    "{} pointer is zero at position {}! Null pointers are not yet supported",
                    name,
                    self.cur_pos,
                )
            }

            bail!(
                "{} pointer misaligned: expected position {} but reader is actually at {} (diff: {})",
                name,
                pointer,
                self.cur_pos,
                i64::from(pointer) - i64::from(self.cur_pos),
            )
        }

        let element = T::deserialize(self)?;
        T::deserialize_post_padding(self, is_last)?;

        Ok(element)
    }

    /// This is called `UndertaleAlignUpdatedListChunk` in UTMT.
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
