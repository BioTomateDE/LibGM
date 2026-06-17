// SPDX-License-Identifier: GPL-3.0-only
//! Not the cleanest code lol

use libgm::prelude::*;
use libgm::wad::elem::texture_page_item::TexturePageItem;

#[derive(Debug, Clone, Default)]
struct Pools {
    tiny: Vec<TexturePageItem>,
    small: Vec<TexturePageItem>,
    medium: Vec<TexturePageItem>,
    big: Vec<TexturePageItem>,
    massive: Vec<TexturePageItem>,
}

enum Size {
    Tiny,
    Small,
    Medium,
    Big,
    Massive,
}

pub fn shuffle_textures(data: &mut GMData) {
    let mut pools = Pools::default();
    let mut sizes = Vec::new();

    for texture in std::mem::take(&mut data.texture_page_items.elems) {
        let size: Size = segregation(&mut pools, texture);
        sizes.push(size);
    }

    shuffle(&mut pools.tiny);
    shuffle(&mut pools.small);
    shuffle(&mut pools.medium);
    shuffle(&mut pools.big);

    for size in sizes {
        let texture: TexturePageItem = match size {
            Size::Tiny => pools.tiny.pop(),
            Size::Small => pools.small.pop(),
            Size::Medium => pools.medium.pop(),
            Size::Big => pools.big.pop(),
            Size::Massive => pools.massive.pop(),
        }
        .unwrap();
        data.texture_page_items.elems.push(texture);
    }
}

fn segregation(pools: &mut Pools, texture: TexturePageItem) -> Size {
    let size = texture.target_width as u32 * texture.target_height as u32;
    if size < 300 {
        pools.tiny.push(texture);
        Size::Tiny
    } else if size < 1200 {
        pools.small.push(texture);
        Size::Small
    } else if size < 8000 {
        pools.medium.push(texture);
        Size::Medium
    } else if size < 20_000 {
        pools.big.push(texture);
        Size::Big
    } else {
        pools.massive.push(texture);
        Size::Massive
    }
}

fn shuffle(vec: &mut Vec<TexturePageItem>) {
    use rand::seq::SliceRandom;
    vec.as_mut_slice().shuffle(&mut rand::rng());
}
