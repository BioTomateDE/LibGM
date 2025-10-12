use crate::gamemaker::deserialize::GMRef;
use crate::gamemaker::elements::GMElement;
use crate::gamemaker::serialize::DataBuilder;
use crate::prelude::*;
use crate::util::fmt::typename;

impl DataBuilder<'_> {
    /// Write the element count as a 32-bit integer.
    /// Then build all elements sequentially, with nothing in between.
    pub fn write_simple_list<T: GMElement>(&mut self, elements: &Vec<T>) -> Result<()> {
        let count: usize = elements.len();
        let ctx = || format!("building simple list of {} with {} elements", typename::<T>(), count,);

        self.write_usize(count).with_context(ctx)?;
        for element in elements {
            element.serialize(self).with_context(ctx)?
        }
        Ok(())
    }

    pub fn write_simple_list_of_resource_ids<T>(&mut self, elements: &Vec<GMRef<T>>) -> Result<()> {
        self.write_usize(elements.len())?;
        for gm_ref in elements {
            self.write_resource_id(gm_ref);
        }
        Ok(())
    }

    pub fn write_simple_list_of_strings(&mut self, elements: &Vec<GMRef<String>>) -> Result<()> {
        let count: usize = elements.len();
        let ctx = || format!("building Simple List of String with {count} elements");

        self.write_usize(count).with_context(ctx)?;
        for gm_string_ref in elements {
            self.write_gm_string(gm_string_ref).with_context(ctx)?;
        }
        Ok(())
    }

    pub fn write_simple_list_short<T: GMElement>(&mut self, elements: &Vec<T>) -> Result<()> {
        let count: usize = elements.len();
        let ctx = || {
            format!(
                "building short simple list of {} with {} elements",
                typename::<T>(),
                count,
            )
        };

        let count: u16 = count
            .try_into()
            .ok()
            .context("Cannot fit element count into 16 bits")
            .with_context(ctx)?;

        self.write_u16(count);
        for element in elements {
            element.serialize(self).with_context(ctx)?;
        }

        Ok(())
    }

    pub fn write_pointer_list<T: GMElement>(&mut self, elements: &Vec<T>) -> Result<()> {
        let count: usize = elements.len();
        let ctx = || format!("building pointer list of {} with {} elements", typename::<T>(), count,);

        self.write_usize(count).with_context(ctx)?;
        let pointer_list_start_pos: usize = self.len();
        for _ in 0..count {
            self.write_u32(0xDEADC0DE);
        }

        for (i, element) in elements.iter().enumerate() {
            element.serialize_pre_padding(self).with_context(ctx)?;
            let resolved_pointer_pos: usize = self.len();
            self.overwrite_usize(resolved_pointer_pos, pointer_list_start_pos + 4 * i)
                .with_context(ctx)?;
            element.serialize(self).with_context(ctx)?;
            element.serialize_post_padding(self, i == count - 1).with_context(ctx)?;
        }
        Ok(())
    }

    /// UndertaleAlignUpdatedListChunk; used for BGND and STRG.
    /// Assumes `chunk.is_aligned`.
    pub fn write_aligned_list_chunk<T: GMElement>(&mut self, elements: &Vec<T>, alignment: u32) -> Result<()> {
        let count: usize = elements.len();
        let ctx = || {
            format!(
                "building aligned chunk pointer list of {} with {} elements",
                typename::<T>(),
                count,
            )
        };

        self.write_usize(count).with_context(ctx)?;
        let pointer_list_start_pos: usize = self.len();
        for _ in 0..count {
            self.write_u32(0xDEADC0DE);
        }

        for (i, element) in elements.iter().enumerate() {
            self.align(alignment);
            let resolved_pointer_pos: usize = self.len();
            self.overwrite_usize(resolved_pointer_pos, pointer_list_start_pos + 4 * i)
                .with_context(ctx)?;
            element.serialize(self).with_context(ctx)?;
        }
        Ok(())
    }
}
