
use image::DynamicImage;
use libgm::prelude::*;

pub fn dump_sprites(data: &GMData) -> Result<()> {
    let dir_name = "sprites";
    std::fs::create_dir_all(dir_name).ctx_any("creating sprites directory")?;

    let mut cached_texture_pages: Vec<Option<DynamicImage>> = vec![None; data.texture_pages.len()];

    for sprite in data.sprites.elements() {
        sprite
            .validate_name(&data.strings)
            .ctx_any("validating sprite name")?;
        let name = data.strings.by_ref(sprite.name)?;

        for (i, &texture_ref) in sprite.textures.iter().enumerate() {
            if texture_ref.is_none() {
                continue;
            }

            let texture = data.texture_page_items.by_ref(texture_ref)?;
            let page_idx = texture.texture_page.index().unwrap();

            let page_image = match &cached_texture_pages[page_idx] {
                Some(cached) => cached,
                None => {
                    let texture_page = data.texture_pages.by_ref(texture.texture_page)?;
                    let gm_image = texture_page.image.as_ref().ok_or(
                        "Sprite's texture is stored in an external texture page which is not yet \
                         supported",
                    )?;
                    let image = gm_image.to_dynamic_image()?;
                    cached_texture_pages[page_idx] = Some((*image).to_owned());
                    cached_texture_pages[page_idx].as_ref().unwrap()
                }
            };

            let cropped_image = page_image.crop_imm(
                texture.source_x as u32,
                texture.source_y as u32,
                texture.source_width as u32,
                texture.source_height as u32,
            );
            cropped_image
                .save(format!("{dir_name}/{name}_{i}.png"))
                .ctx_any("saving sprite image")?;
        }
    }

    Ok(())
}
