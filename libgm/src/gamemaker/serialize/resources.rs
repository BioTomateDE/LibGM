use crate::{
    gamemaker::{
        elements::{strings::StringPlaceholder, texture_page_items::GMTexturePageItem},
        reference::GMRef,
        serialize::builder::DataBuilder,
    },
    prelude::*,
};

impl DataBuilder<'_> {
    /// Writes the resource ID (index) from a `GMRef`.
    /// # Parameters
    /// - `resource`: The resource reference whose index to write.
    pub fn write_resource_id<T>(&mut self, resource: GMRef<T>) {
        self.write_u32(resource.index);
    }

    /// Writes the resource ID if present; writes -1 if `None`.
    /// # Parameters
    /// - `resource`: Optional resource reference to write.
    pub fn write_resource_id_opt<T>(&mut self, resource: Option<GMRef<T>>) {
        match resource {
            Some(gm_ref) => self.write_u32(gm_ref.index),
            None => self.write_i32(-1),
        }
    }

    fn write_gm_string_internal(&mut self, string: String, write_id: bool) {
        let placeholder_position = self.len() as u32;
        let placeholder = StringPlaceholder { placeholder_position, string, write_id };
        self.string_placeholders.push(placeholder);
        self.write_u32(0xDEAD_C0DE);
    }

    /// Writes a GameMaker string reference as a pointer placeholder.
    pub fn write_gm_string(&mut self, string: &str) {
        self.write_gm_string_internal(string.to_string(), false);
    }

    /// Writes an optional GameMaker string reference as a pointer placeholder, or zero if the reference is `None`.
    pub fn write_gm_string_opt(&mut self, string_opt: &Option<String>) {
        match string_opt {
            Some(string) => self.write_gm_string(string),
            None => self.write_u32(0),
        }
    }

    /// Writes a GameMaker string reference as a String ID/Index.
    pub fn write_gm_string_id(&mut self, string: String) {
        self.write_gm_string_internal(string, true);
    }

    /// Writes a GameMaker texture page item reference as a pointer placeholder.
    /// # Errors
    /// Returns an error if the contained texture page item reference cannot be resolved.
    pub fn write_gm_texture(&mut self, gm_texture_ref: GMRef<GMTexturePageItem>) -> Result<()> {
        let elem: &GMTexturePageItem = self.gm_data.texture_page_items.by_ref(gm_texture_ref)?;
        self.write_pointer(elem);
        Ok(())
    }

    /// Writes an optional GameMaker texture page item reference as a pointer placeholder, or zero if the reference is `None`.
    /// # Errors
    /// Returns an error if the contained texture page item reference cannot be resolved.
    pub fn write_gm_texture_opt(
        &mut self,
        gm_texture_ref_opt: Option<GMRef<GMTexturePageItem>>,
    ) -> Result<()> {
        match gm_texture_ref_opt {
            Some(gm_texture_ref) => self.write_gm_texture(gm_texture_ref)?,
            None => self.write_u32(0),
        }
        Ok(())
    }
}
