use crate::prelude::*;
use crate::gamemaker::deserialize::GMRef;
use crate::gamemaker::elements::texture_page_items::GMTexturePageItem;
use crate::gamemaker::serialize::DataBuilder;

impl DataBuilder<'_> {
    /// Writes the resource ID (index) from a `GMRef`.
    /// # Parameters
    /// - `resource`: The resource reference whose index to write.
    pub fn write_resource_id<T>(&mut self, resource: &GMRef<T>) {
        self.write_u32(resource.index);
    }

    /// Writes the resource ID if present; writes -1 if `None`.
    /// # Parameters
    /// - `resource`: Optional resource reference to write.
    pub fn write_resource_id_opt<T>(&mut self, resource: &Option<GMRef<T>>) {
        match resource {
            Some(gm_ref) => self.write_u32(gm_ref.index),
            None => self.write_i32(-1),
        }
    }

    /// Writes a GameMaker string reference as a pointer placeholder.
    /// # Errors
    /// Returns an error if the contained string reference cannot be resolved.
    pub fn write_gm_string(&mut self, gm_string_ref: &GMRef<String>) -> Result<()> {
        let resolved_string: &String = gm_string_ref.resolve(&self.gm_data.strings.strings)?;
        self.write_pointer(resolved_string)?;
        Ok(())
    }

    /// Writes an optional GameMaker string reference as a pointer placeholder, or zero if the reference is `None`.
    /// # Errors
    /// Returns an error if the contained string reference cannot be resolved.
    pub fn write_gm_string_opt(&mut self, gm_string_ref_opt: &Option<GMRef<String>>) -> Result<()> {
        match gm_string_ref_opt {
            Some(string_ref) => self.write_gm_string(string_ref)?,
            None => self.write_u32(0),
        }
        Ok(())
    }

    /// Writes a GameMaker texture page item reference as a pointer placeholder.
    /// # Errors
    /// Returns an error if the contained texture page item reference cannot be resolved.
    pub fn write_gm_texture(&mut self, gm_texture_ref: &GMRef<GMTexturePageItem>) -> Result<()> {
        let resolved_texture_page_item: &GMTexturePageItem = gm_texture_ref.resolve(&self.gm_data.texture_page_items.texture_page_items)?;
        self.write_pointer(resolved_texture_page_item)
    }

    /// Writes an optional GameMaker texture page item reference as a pointer placeholder, or zero if the reference is `None`.
    /// # Errors
    /// Returns an error if the contained texture page item reference cannot be resolved.
    pub fn write_gm_texture_opt(&mut self, gm_texture_ref_opt: &Option<GMRef<GMTexturePageItem>>) -> Result<()> {
        match gm_texture_ref_opt {
            Some(gm_texture_ref) => self.write_gm_texture(gm_texture_ref)?,
            None => self.write_u32(0),
        }
        Ok(())
    }

    /// Resolves a GameMaker string reference to the actual character string.
    /// Returns an error if the reference index is out of bounds.
    pub fn resolve_gm_str(&self, gm_string_ref: &GMRef<String>) -> Result<&String> {
        gm_string_ref.resolve(&self.gm_data.strings.strings)
    }

    /// Tries to resolve a GameMaker string reference to the actual character string.
    /// Returns a placeholder string if resolving failed.
    /// 
    /// This function is meant to be used in closures where propagating errors is awkward.
    /// Otherwise, using [`DataBuilder::resolve_gm_str`] is preferred.
    pub fn display_gm_str(&self, gm_string_ref: &GMRef<String>) -> &str {
        gm_string_ref.display(&self.gm_data.strings)
    }
}

