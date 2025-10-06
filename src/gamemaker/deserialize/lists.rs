use crate::prelude::*;
use crate::gamemaker::deserialize::{DataReader, GMRef};
use crate::gamemaker::elements::GMElement;
use crate::util::fmt::{format_bytes, typename};

impl DataReader<'_> {
    /// Reads a GameMaker simple list by calling the specified deserializer function for each element.
    ///
    /// Simple lists consist of a count followed by the elements' data in sequence.
    /// Includes a failsafe check to prevent excessive memory allocation from malformed data.
    fn read_simple_list_internal<T>(&mut self, count: usize, deserializer_fn: impl Fn(&mut Self) -> Result<T>) -> Result<Vec<T>> {
        const FAILSAFE_SIZE: usize = 1_000_000;   // 1 Megabyte
        let implied_data_size: usize = count * size_of::<T>();
        if implied_data_size > FAILSAFE_SIZE {
            bail!(
                "Failsafe triggered in chunk '{}' at position {} while trying \
                to read simple list of {}: Element count {} implies a total data \
                size of {} which is larger than the failsafe size of {}",
                self.chunk.name, self.cur_pos-4, typename::<T>(),
                count, format_bytes(implied_data_size), format_bytes(FAILSAFE_SIZE),
            );
        }
        let mut elements: Vec<T> = Vec::with_capacity(count);
        for _ in 0..count {
            let element: T = deserializer_fn(self).with_context(|| format!(
                "while deserializing element #{}/{} of {} simple list",
                elements.len(), count, typename::<T>(),
            ))?;
            elements.push(element);
        }
        Ok(elements)
    }

    /// Reads a GameMaker simple list with a 32-bit count prefix.
    ///
    /// The list format is: `[count: u32][element_0][element_1]...[element_n]`
    pub fn read_simple_list<T: GMElement>(&mut self) -> Result<Vec<T>> {
        let count = self.read_usize()?;
        self.read_simple_list_internal(count, T::deserialize)
    }

    /// Reads a GameMaker simple list with a 16-bit count prefix.
    ///
    /// The list format is: `[count: u16][element_0][element_1]...[element_n]`
    /// Uses a smaller failsafe limit appropriate for short lists.
    pub fn read_simple_list_short<T: GMElement>(&mut self) -> Result<Vec<T>> {
        let count = self.read_u16()?;
        self.read_simple_list_internal(count as usize, T::deserialize)
    }

    /// Reads a simple list of resource IDs and wraps them in [`GMRef`].
    ///
    /// Each element is a 32-bit resource ID that gets resolved to a reference.
    pub fn read_simple_list_of_resource_ids<T/*: GMElement*/>(&mut self) -> Result<Vec<GMRef<T>>> {
        let count = self.read_usize()?;
        self.read_simple_list_internal(count, |reader| reader.read_resource_by_id())
    }


    /// Reads a simple list of GameMaker string references.
    pub fn read_simple_list_of_strings(&mut self) -> Result<Vec<GMRef<String>>> {
        let count = self.read_usize()?;
        self.read_simple_list_internal(count, |reader| reader.read_gm_string())
    }

    pub fn read_pointer_list<T: GMElement>(&mut self) -> Result<Vec<T>> {
        // TODO implement 2024.11+ null pointers (unused asset removal)
        let pointers: Vec<u32> = self.read_simple_list()?;
        let count: usize = pointers.len();

        let mut elements: Vec<T> = Vec::with_capacity(count);
        for (i, pointer) in pointers.into_iter().enumerate() {
            let element: T = self.read_pointer_element(pointer, i == count-1).with_context(|| format!(
                "deserializing element #{}/{} of {} pointer list",
                elements.len(), count, typename::<T>(),
            ))?;
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

    /// UndertaleAlignUpdatedListChunk; used for BGND and STRG
    pub fn read_aligned_list_chunk<T: GMElement>(&mut self, alignment: u32, is_aligned: &mut bool) -> Result<Vec<T>> {
        let pointers: Vec<u32> = self.read_simple_list()?;
        let mut elements: Vec<T> = Vec::with_capacity(pointers.len());

        for pointer in &pointers {
            if pointer % alignment != 0 {
                *is_aligned = false;
            }
            if *pointer == 0 {
                // can happen in 2024.11+ (unused assets removal)
                bail!("Null pointers are not yet supported while parsing aligned list chunk");
            }
        }

        for pointer in pointers {
            if *is_aligned {
                self.align(alignment)?;
            }
            self.assert_pos(pointer, "Aligned list chunk")?;    // UTMT doesn't do this afaik
            let element = T::deserialize(self)?;
            elements.push(element);
        }
        Ok(elements)
    }
}

