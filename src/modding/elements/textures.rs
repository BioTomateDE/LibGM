use std::borrow::Cow;
use std::collections::HashMap;
use std::iter::zip;
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use crate::gamemaker::elements::texture_page_items::GMTexturePageItem;
use crate::modding::export::{edit_field, ModExporter, ModRef};
use crate::modding::unordered_list::EditUnorderedList;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTexturePageItem {
    // texture is stored as a png; referenced by {index}.png
    pub image: ModRef,   // image ref
    pub target_x: u16,
    pub target_y: u16,
    pub target_width: u16,
    pub target_height: u16,
    pub bounding_width: u16,
    pub bounding_height: u16,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditTexturePageItem {
    pub image: Option<ModRef>,   // image ref
    pub target_x: Option<u16>,
    pub target_y: Option<u16>,
    pub target_width: Option<u16>,
    pub target_height: Option<u16>,
    pub bounding_width: Option<u16>,
    pub bounding_height: Option<u16>,
}

impl ModExporter<'_, '_> {
    pub fn export_textures(
        &self,
        original_images: Vec<Option<Cow<DynamicImage>>>,
        modified_images: Vec<Option<Cow<DynamicImage>>>,
    ) -> Result<(EditUnorderedList<AddTexturePageItem, EditTexturePageItem>, Vec<DynamicImage>), String> {
        // export_changes_unordered_list(
        let original_list: &Vec<GMTexturePageItem> = &self.original_data.texture_page_items.texture_page_items;
        let modified_list: &Vec<GMTexturePageItem> = &self.modified_data.texture_page_items.texture_page_items;
        
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
            let texture_page_img: &Cow<DynamicImage> = &modified_images[i.texture_page.index as usize].as_ref().unwrap();
            let cropped_image: DynamicImage = crop_from_texture_page(texture_page_img, i)?;
            
            let add_texture_page_item = AddTexturePageItem {
                image: ModRef::Add(textures.len() as u32),
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
            let original_texture_page_img: &Cow<DynamicImage> = &original_images[original.texture_page.index as usize].as_ref().unwrap();
            let modified_texture_page_img: &Cow<DynamicImage> = &modified_images[modified.texture_page.index as usize].as_ref().unwrap();
            let original_cropped: DynamicImage = crop_from_texture_page(original_texture_page_img, original)?;  // <-- slow operation
            let modified_cropped: DynamicImage = crop_from_texture_page(modified_texture_page_img, modified)?;  // <-- slow operation

            // TODO: optimize
            let image_unchanged: bool = original_cropped == modified_cropped;   // <-- slow operation
            if image_unchanged
                && original.target_x == modified.target_x
                && original.target_y == modified.target_y
                && original.target_width == modified.target_width
                && original.target_height == modified.target_height
                && original.bounding_width == modified.bounding_width
                && original.bounding_height == modified.bounding_height {
                continue
            }
            
            // it will always create a new cropped image; even if the texture is also in the original
            let edit = EditTexturePageItem {
                image: Some(ModRef::Add(textures.len() as u32)),
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


fn crop_from_texture_page(texture_page_img: &Cow<DynamicImage>, texture_page_item: &GMTexturePageItem) -> Result<DynamicImage, String> {
    let cropped_image: DynamicImage = texture_page_img.crop_imm(
        texture_page_item.source_x as u32,
        texture_page_item.source_y as u32,
        texture_page_item.source_width as u32,
        texture_page_item.source_height as u32,
    );  // <--- slow operation
    Ok(cropped_image)
}

