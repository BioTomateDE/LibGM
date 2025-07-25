use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::iter::zip;
use image::{DynamicImage, RgbaImage, SubImage};
use serde::{Deserialize, Serialize};
use crate::gamemaker::elements::embedded_textures::GMEmbeddedTexture;
use crate::gamemaker::elements::texture_page_items::GMTexturePageItem;
use crate::modding::export::{edit_field, ModExporter, ModRef};
use crate::utility::Stopwatch;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModTexturePageItem {
    // texture is stored as a png; referenced by {index}.png
    pub image: ModRef,   // image ref
    pub target_x: u16,
    pub target_y: u16,
    pub target_width: u16,
    pub target_height: u16,
    pub bounding_width: u16,
    pub bounding_height: u16,
}


impl ModExporter<'_, '_> {
    pub fn export_textures(&self) -> Result<(Vec<ModTexturePageItem>, Vec<DynamicImage>), String> {
        let original_list: &Vec<GMTexturePageItem> = &self.original_data.texture_page_items.texture_page_items;
        let modified_list: &Vec<GMTexturePageItem> = &self.modified_data.texture_page_items.texture_page_items;

        // get `RgbaImage`s for each texture page (original & modified)
        let stopwatch = Stopwatch::start();
        let original_images: Vec<Option<Cow<RgbaImage>>> = get_images(&self.original_data.embedded_textures.texture_pages)?;
        let modified_images: Vec<Option<Cow<RgbaImage>>> = get_images(&self.modified_data.embedded_textures.texture_pages)?;
        log::trace!("Getting {} rgba images took {stopwatch}", original_images.len() + modified_images.len());

        // Hash original Texture Pages' images for fast lookups
        let mut original_image_hashes = HashSet::with_capacity(original_images.len());
        for texture_page_item in original_list {
            let Some(full_img) = original_images[texture_page_item.texture_page.index as usize].as_ref()
            else { continue };
            let cropped_image: RgbaImage = crop_from_texture_page(full_img, texture_page_item)?;
            let bytes: &Vec<u8> = cropped_image.as_raw();
            let hash: [u8; 16] = xxhash_rust::xxh3::xxh3_128(bytes).to_le_bytes();
            let unique: bool = original_image_hashes.insert(hash); 
            if !unique {
                return Err(
                    "Congratulations! You achieved a 1 in 10^38 chance of a hash collision! \
                    With that luck, you could've easily won the lottery dozens of times...".to_string()
                )
            }
        }

        // TODO: changes to target x, y, width, height as well as bounding width and height are currently not detectable
        let mut new_texture_page_items = Vec::new();

        for (i, texture_page_item) in modified_list.iter().enumerate() {
            let full_img: &Cow<RgbaImage> = modified_images[texture_page_item.texture_page.index as usize].as_ref()
                .ok_or("External texture pages are not yet supported")?;    // TODO
            let cropped_image: RgbaImage = crop_from_texture_page(full_img, texture_page_item)?;
            let bytes: &Vec<u8> = cropped_image.as_raw();
            let hash: [u8; 16] = xxhash_rust::xxh3::xxh3_128(bytes).to_le_bytes();
            if original_image_hashes.get(&hash).is_some() {
                continue    // image already existed in original data; continue
            }
            
            // image was newly added!
            let new_texture_page_item = ModTexturePageItem {
                image: ModRef::Add(textures.len() as u32),
                target_x: i.target_x,
                target_y: i.target_y,
                target_width: i.target_width,
                target_height: i.target_height,
                bounding_width: i.bounding_width,
                bounding_height: i.bounding_height,
            };
            new_texture_page_items.push()
        }

// OLD        
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
            
            let add_texture_page_item = ModTexturePageItem {
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

        Ok((EditOrderedList { additions, edits }, textures))
    }
}


fn crop_from_texture_page(texture_page_img: &Cow<RgbaImage>, texture_page_item: &GMTexturePageItem) -> Result<RgbaImage, String> {
    let source_img: &RgbaImage = texture_page_img.as_ref();
    let x: u32 = texture_page_item.source_x as u32;
    let y: u32 = texture_page_item.source_y as u32;
    let w: u32 = texture_page_item.source_width as u32;
    let h: u32 = texture_page_item.source_height as u32;

    let cropped_image: SubImage<&RgbaImage> = image::imageops::crop_imm(source_img, x, y, w, h);
    let rgba_image: RgbaImage = cropped_image.to_image();
    Ok(rgba_image)
}


fn get_images(texture_pages: &[GMEmbeddedTexture]) -> Result<Vec<Option<Cow<RgbaImage>>>, String> {
    use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
    texture_pages
        .par_iter()
        .map(|texture_page| {
            texture_page.image
                .as_ref()
                .map(|gm_image| {
                    let dynamic_image: Cow<DynamicImage> = gm_image.to_dynamic_image()?;
                    let rgba_image: Cow<RgbaImage> = match dynamic_image {
                        Cow::Owned(DynamicImage::ImageRgba8(rgba_img)) => Cow::Owned(rgba_img),
                        Cow::Borrowed(DynamicImage::ImageRgba8(rgba_img)) => Cow::Borrowed(rgba_img),
                        Cow::Owned(dyn_img) => Cow::Owned(dyn_img.to_rgba8()),
                        Cow::Borrowed(dyn_img) => Cow::Owned(dyn_img.to_rgba8()),
                    };
                    Ok(rgba_image)
                })
                .transpose()
        }).collect()
}

