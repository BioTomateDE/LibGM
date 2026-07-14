use libgm::prelude::*;

pub fn dump_texture_pages(data: &GMData) -> Result<()> {
    let dirname = "texturepages";
    std::fs::create_dir_all(dirname).ctx_any("creating texture pages directory")?;

    for (i, txtr) in data.texture_pages.elements().enumerate() {
        let Some(img) = &txtr.image else {
            continue;
        };
        let img = img.to_dynamic_image()?;
        img.save(format!("{dirname}/{i}.png"))
            .ctx_any("saving texture page")?;
    }

    Ok(())
}
