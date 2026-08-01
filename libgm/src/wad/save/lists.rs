// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::util::fmt::typename;
use crate::wad::elem::GMComplexElement;
use crate::wad::elem::GMElement;
use crate::wad::save::writer::DataWriter;

impl DataWriter<'_> {
    /// Writes the element count as a 32-bit integer.
    /// Then serializes all elements sequentially, with nothing in between.
    pub fn write_simple_list_ctx<T: GMComplexElement>(
        &mut self,
        elements: &Vec<T>,
        write_context: T::WriteContext,
    ) -> Result<()> {
        let count: usize = elements.len();
        let ctx =
            || format!("building simple list of {} with {} elements", typename::<T>(), count,);

        self.write_usize(count).ctx(ctx)?;
        for element in elements {
            element.serialize_complex(self, write_context).ctx(ctx)?;
        }
        Ok(())
    }

    #[inline]
    pub fn write_simple_list<T: GMElement>(&mut self, elements: &Vec<T>) -> Result<()> {
        self.write_simple_list_ctx(elements, ())
    }

    pub fn write_simple_list_short<T: GMElement>(&mut self, elements: &Vec<T>) -> Result<()> {
        let count: usize = elements.len();
        let ctx = || {
            format!("building short simple list of {} with {} elements", typename::<T>(), count,)
        };

        let count: u16 =
            count.try_into().map_err(|_| "Cannot fit element count into 16 bits").ctx(ctx)?;

        self.write_u16(count);
        for element in elements {
            element.serialize(self).ctx(ctx)?;
        }

        Ok(())
    }

    pub fn write_pointer_list_ctx<T: GMComplexElement>(
        &mut self,
        elements: &[T],
        write_context: T::WriteContext,
    ) -> Result<()> {
        let count: usize = elements.len();
        let ctx =
            || format!("building pointer list of {} with {} elements", typename::<T>(), count,);

        self.write_usize(count).ctx(ctx)?;
        let pointer_list_pos: u32 = self.pos();
        for _ in 0..count {
            self.write_u32(0xDEAD_C0DE);
        }

        for (i, element) in elements.iter().enumerate() {
            self.align(T::PRE_PADDING);
            self.patch_pointer_list(pointer_list_pos, i).ctx(ctx)?;
            element.serialize_complex(self, write_context).ctx(ctx)?;
            self.align(T::POST_PADDING);
        }
        Ok(())
    }

    #[inline]
    pub fn write_pointer_list<T: GMElement>(&mut self, elements: &[T]) -> Result<()> {
        self.write_pointer_list_ctx(elements, ())
    }

    // TODO: clean up this code
    pub fn write_pointer_list_opt_ctx<T: GMComplexElement>(
        &mut self,
        elements: &[Option<T>],
        write_context: T::WriteContext,
    ) -> Result<()> {
        let count: usize = elements.len();
        let ctx = || {
            format!("building nullable pointer list of {} with {} elements", typename::<T>(), count,)
        };

        self.write_usize(count).ctx(ctx)?;
        let pointer_list_pos: u32 = self.pos();
        for _ in 0..count {
            self.write_u32(0);
        }

        for (i, element_opt) in elements.iter().enumerate() {
            let Some(element) = element_opt else { continue };
            self.align(T::PRE_PADDING);
            self.patch_pointer_list(pointer_list_pos, i).ctx(ctx)?;
            element.serialize_complex(self, write_context).ctx(ctx)?;
            self.align(T::POST_PADDING);
        }
        Ok(())
    }

    #[inline]
    pub fn write_pointer_list_opt<T: GMElement>(&mut self, elements: &[Option<T>]) -> Result<()> {
        self.write_pointer_list_opt_ctx(elements, ())
    }
}
