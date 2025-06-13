use std::collections::HashMap;
use std::iter::zip;
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use crate::deserialize::embedded_textures::GMEmbeddedTexture;
use crate::deserialize::texture_page_items::GMTexturePageItem;
use crate::export_mod::export::{edit_field, ModExporter, ModRef};
use crate::export_mod::unordered_list::EditUnorderedList;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTexturePageItem {
    // texture is stored as a png; referenced by {index}.png
    pub texture_page: ModRef,   //???
    pub target_x: u16,
    pub target_y: u16,
    pub target_width: u16,
    pub target_height: u16,
    pub bounding_width: u16,
    pub bounding_height: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditTexturePageItem {
    pub texture_page: Option<ModRef>,   //???
    pub target_x: Option<u16>,
    pub target_y: Option<u16>,
    pub target_width: Option<u16>,
    pub target_height: Option<u16>,
    pub bounding_width: Option<u16>,
    pub bounding_height: Option<u16>,
}

impl ModExporter<'_, '_> {
    pub fn export_textures(&self) -> Result<(EditUnorderedList<AddTexturePageItem, EditTexturePageItem>, Vec<DynamicImage>), String> {
        // export_changes_unordered_list(
        let original_list: &Vec<GMTexturePageItem> = &self.original_data.texture_page_items.textures_by_index;
        let modified_list: &Vec<GMTexturePageItem> = &self.modified_data.texture_page_items.textures_by_index;
        
        let texture_page_items: &[GMTexturePageItem] = modified_list
            .get(original_list.len() .. modified_list.len())
            .ok_or_else(|| format!(
                "Could not get texture additions slice with original data len {} and modified data len {}. \
                If there are purposefully fewer texture page items in your modified data file, please report this as a bug.",
                original_list.len(), modified_list.len(),
            ))?;
        
        let mut textures: Vec<DynamicImage> = Vec::with_capacity(texture_page_items.len());
        let mut additions = Vec::with_capacity(texture_page_items.len());
        
        for i in texture_page_items {
            // TODO handle GMEmbeddedTexture fields like scaled or generated_mips
            let texture_page: &GMEmbeddedTexture = i.texture_page.resolve(&self.modified_data.texture_pages)?;
            let cropped_image: DynamicImage = crop_from_texture_page(texture_page, i)?;
            
            let add_texture_page_item = AddTexturePageItem {
                texture_page: ModRef::Add(textures.len()),
                target_x: i.target_x,
                target_y: i.target_y,
                target_width: i.target_width,
                target_height: i.target_height,
                bounding_width: i.bounding_width,
                bounding_height: i.bounding_height,
            };
            textures.push(cropped_image);   // it will always create a new cropped image; even if the texture is also in the original
            additions.push(add_texture_page_item);
        }

        let mut edits: HashMap<usize, EditTexturePageItem> = HashMap::new();
        for (i, (original, modified)) in zip(original_list, modified_list).enumerate() {
            let original_texture_page: &GMEmbeddedTexture = original.texture_page.resolve(&self.original_data.texture_pages)?;
            let modified_texture_page: &GMEmbeddedTexture = modified.texture_page.resolve(&self.modified_data.texture_pages)?;
            let original_cropped: DynamicImage = crop_from_texture_page(original_texture_page, original)?;
            let modified_cropped: DynamicImage = crop_from_texture_page(modified_texture_page, modified)?;

            if original_cropped == modified_cropped
                && original.target_x == modified.target_x
                && original.target_y == modified.target_y
                && original.target_width == modified.target_width
                && original.target_height == modified.target_height
                && original.bounding_width == modified.bounding_width
                && original.bounding_height == modified.bounding_height {
                continue
            }
            
            // TODO it will always create a new cropped image; even if the texture is also in the original
            let edit = EditTexturePageItem {
                texture_page: Some(ModRef::Add(textures.len())),
                target_x: edit_field(&original.target_x, &modified.target_x),
                target_y: edit_field(&original.target_y, &modified.target_y),
                target_width: edit_field(&original.target_width, &modified.target_width),
                target_height: edit_field(&original.target_height, &modified.target_height),
                bounding_width: edit_field(&original.bounding_width, &modified.bounding_width),
                bounding_height: edit_field(&original.bounding_height, &modified.bounding_height),
            };
            textures.push(modified_cropped);
            edits.insert(i, edit);
        }

        Ok((EditUnorderedList { additions, edits }, textures))
    }
}


fn crop_from_texture_page(texture_page: &GMEmbeddedTexture, texture_page_item: &GMTexturePageItem) -> Result<DynamicImage, String> {
    let texture_page_image: &DynamicImage = texture_page.image.as_ref()
        .ok_or_else(|| format!("Image not set for texture page #{}; external textures pages are not yet supported", texture_page_item.texture_page.index))?;
    
    let cropped_image: DynamicImage = texture_page_image.crop_imm(
        texture_page_item.source_x as u32,
        texture_page_item.source_y as u32,
        texture_page_item.source_width as u32,
        texture_page_item.source_height as u32,
    );
    
    Ok(cropped_image)
}

