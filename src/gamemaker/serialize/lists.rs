use crate::gamemaker::deserialize::GMRef;
use crate::gamemaker::elements::GMElement;
use crate::gamemaker::serialize::DataBuilder;
use crate::utility::typename;

impl DataBuilder<'_> {
    /// Write the element count as a 32-bit integer.
    /// Then build all elements sequentially, with nothing in between.
    pub fn write_simple_list<T: GMElement>(&mut self, elements: &Vec<T>) -> Result<(), String> {
        let count: usize = elements.len();
        self.write_usize(count)?;
        for element in elements {
            element.serialize(self).map_err(|e| format!(
                "{e}\n↳ while building simple list of {} with {} elements",
                typename::<T>(), count,
            ))?;
        }
        Ok(())
    }

    pub fn write_simple_list_of_resource_ids<T>(&mut self, elements: &Vec<GMRef<T>>) -> Result<(), String> {
        self.write_usize(elements.len())?;
        for gm_ref in elements {
            self.write_resource_id(gm_ref);
        }
        Ok(())
    }

    pub fn write_simple_list_of_strings(&mut self, elements: &Vec<GMRef<String>>) -> Result<(), String> {
        let count: usize = elements.len();
        self.write_usize(count)?;
        for gm_string_ref in elements {
            self.write_gm_string(gm_string_ref)
                .map_err(|e| format!("{e}\n↳ while building simple list of String with {count} elements"))?;
        }
        Ok(())
    }

    pub fn write_simple_list_short<T: GMElement>(&mut self, elements: &Vec<T>) -> Result<(), String> {
        let count: usize = elements.len();
        let count: u16 = count.try_into().map_err(|_| format!(
            "Error while building short simple list with {count} elements: cannot fit element count into 16 bits",
        ))?;
        self.write_u16(count);
        for element in elements {
            element.serialize(self).map_err(|e| format!(
                "{e}\n↳ while building short simple list of {} with {} elements",
                typename::<T>(), count,
            ))?;
        }
        Ok(())
    }

    pub fn write_pointer_list<T: GMElement>(&mut self, elements: &Vec<T>) -> Result<(), String> {
        let count: usize = elements.len();
        self.write_usize(count)?;
        let pointer_list_start_pos: usize = self.len();
        for _ in 0..count {
            self.write_u32(0xDEADC0DE);
        }

        for (i, element) in elements.iter().enumerate() {
            element.serialize_pre_padding(self)?;
            let resolved_pointer_pos: usize = self.len();
            self.overwrite_usize(resolved_pointer_pos, pointer_list_start_pos + 4*i)?;
            element.serialize(self).map_err(|e| format!(
                "{e}\n↳ while building pointer list of {} with {} elements",
                typename::<T>(), count,
            ))?;
            element.serialize_post_padding(self, i == count-1)?;
        }
        Ok(())
    }

    /// UndertaleAlignUpdatedListChunk; used for BGND and STRG.
    /// Assumes `chunk.is_aligned`.
    /// TODO: copypasted ass function
    pub fn write_aligned_list_chunk<T: GMElement>(&mut self, elements: &Vec<T>, alignment: usize) -> Result<(), String> {
        let count: usize = elements.len();
        self.write_usize(count)?;
        let pointer_list_start_pos: usize = self.len();
        for _ in 0..count {
            self.write_u32(0xDEADC0DE);
        }

        for (i, element) in elements.iter().enumerate() {
            self.align(alignment);
            let resolved_pointer_pos: usize = self.len();
            self.overwrite_usize(resolved_pointer_pos, pointer_list_start_pos + 4*i)?;
            element.serialize(self).map_err(|e| format!(
                "{e}\n↳ while building aligned chunk pointer list of {} with {} elements",
                typename::<T>(), count,
            ))?;
        }
        Ok(())
    }
}

