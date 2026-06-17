// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::texture_page_item::TexturePageItem;
use crate::wad::reference::GMRef;

impl DataBuilder<'_> {
    /// Writes the resource ID (index) from a `GMRef`.
    ///
    /// # Parameters
    /// - `resource`: The resource reference whose index to write.
    pub fn write_resource_id<T>(&mut self, resource: GMRef<T>) {
        self.write_i32(resource.index);
    }

    /// Writes a GameMaker string reference as a pointer placeholder.
    pub fn write_gm_string(&mut self, string_ref: GMRef<String>) -> Result<()> {
        if string_ref.is_some() {
            let elem: &String = self.gm_data.strings.by_ref(string_ref)?;
            self.write_pointer(elem);
        } else {
            self.write_i32(0);
        }
        Ok(())
    }

    /// Writes a GameMaker texture page item reference as a pointer placeholder.
    ///
    /// # Errors
    /// Returns an error if the contained texture page item reference cannot be
    /// resolved.
    pub fn write_gm_texture(&mut self, gm_texture_ref: GMRef<TexturePageItem>) -> Result<()> {
        if gm_texture_ref.is_some() {
            let elem: &TexturePageItem = self.gm_data.texture_page_items.by_ref(gm_texture_ref)?;
            self.write_pointer(elem);
        } else {
            self.write_i32(0);
        }
        Ok(())
    }
}
