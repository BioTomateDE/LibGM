use crate::gamemaker::deserialize::{DataReader, GMRef};
use crate::gamemaker::elements::GMElement;
use crate::utility::{format_bytes, typename};

impl DataReader<'_> {
    /// Reads a GameMaker simple list by calling the specified `deserializer_fn` for every element.
    /// Simple lists consists of the element count, followed by the elements' data.
    fn read_simple_list_internal<T>(&mut self, deserializer_fn: impl Fn(&mut Self) -> Result<T, String>) -> Result<Vec<T>, String> {
        const FAILSAFE_SIZE: usize = 1_000_000;   // 1 Megabyte
        let count: usize = self.read_usize()?;
        let implied_data_size: usize = count * size_of::<T>();
        if implied_data_size > FAILSAFE_SIZE {
            return Err(format!(
                "Failsafe triggered in chunk '{}' at position {} while trying \
                to read simple list of {}: Element count {} implies a total data \
                size of {} which is larger than the failsafe size of {}",
                self.chunk.name, self.cur_pos-4, typename::<T>(),
                count, format_bytes(implied_data_size), format_bytes(FAILSAFE_SIZE),
            ))
        }
        let mut elements: Vec<T> = Vec::with_capacity(count);
        for _ in 0..count {
            let element: T = deserializer_fn(self).map_err(|e| format!(
                "{e}\n↳ while deserializing element #{}/{} of {} simple list",
                elements.len(), count, typename::<T>(),
            ))?;
            elements.push(element);
        }
        Ok(elements)
    }
    
    /// Read a GameMaker simple list.
    pub fn read_simple_list<T: GMElement>(&mut self) -> Result<Vec<T>, String> {
        self.read_simple_list_internal(T::deserialize)
    }

    pub fn read_simple_list_of_resource_ids<T/*: GMElement*/>(&mut self) -> Result<Vec<GMRef<T>>, String> {
        self.read_simple_list_internal(|reader| reader.read_resource_by_id())
    }

    pub fn read_simple_list_of_strings(&mut self) -> Result<Vec<GMRef<String>>, String> {
        self.read_simple_list_internal(|reader| reader.read_gm_string())
    }

    /// this could probably be moved to gmkerning; it doesn't seem to be used anywhere else
    pub fn read_simple_list_short<T: GMElement>(&mut self) -> Result<Vec<T>, String> {
        const FAILSAFE_SIZE: usize = 10_000;   // 10 Kilobytes
        let count: usize = self.read_u16()? as usize;
        let implied_data_size: usize = count * size_of::<T>();
        if implied_data_size > FAILSAFE_SIZE {
            return Err(format!(
                "Failsafe triggered in chunk '{}' at position {} while trying \
                to read short simple list of {}: Element count {} implies a total data \
                size of {} which is larger than the failsafe size of {}",
                self.chunk.name, self.cur_pos-4, typename::<T>(),
                count, format_bytes(implied_data_size), format_bytes(FAILSAFE_SIZE),
            ))
        }
        let mut elements: Vec<T> = Vec::with_capacity(count);
        for _ in 0..count {
            elements.push(T::deserialize(self)?);
        }
        Ok(elements)
    }

    pub fn read_pointer_list<T: GMElement>(&mut self) -> Result<Vec<T>, String> {
        // TODO implement 2024.11+ null pointers (unused asset removal)
        let pointers: Vec<usize> = self.read_simple_list()?;
        let count: usize = pointers.len();

        let mut elements: Vec<T> = Vec::with_capacity(count);
        for (i, pointer) in pointers.into_iter().enumerate() {
            // note: this scuffed closure is only used to prevent repetition in map_err.
            //       it will be replaced when try blocks are added to stable.
            let element: T = (|| {
                T::deserialize_pre_padding(self)?;
                self.assert_pos(pointer, &typename::<T>())?;
                let element = T::deserialize(self)?;
                T::deserialize_post_padding(self, i == count-1)?;
                Ok(element)
            })().map_err(|e: String| format!(
                "{e}\n↳ while deserializing element #{}/{} of {} pointer list",
                elements.len(), count, typename::<T>(),
            ))?;
            elements.push(element);
        }
        Ok(elements)
    }

    /// UndertaleAlignUpdatedListChunk; used for BGND and STRG
    pub fn read_aligned_list_chunk<T: GMElement>(&mut self, alignment: usize, is_aligned: &mut bool) -> Result<Vec<T>, String> {
        let pointers: Vec<usize> = self.read_simple_list()?;
        let mut elements: Vec<T> = Vec::with_capacity(pointers.len());

        for pointer in &pointers {
            if pointer % alignment != 0 {
                *is_aligned = false;
            }
            if *pointer == 0 {
                // can happen in 2024.11+ (unused assets removal)
                return Err("Null pointers are not yet supported while parsing aligned list chunk".to_string())
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

