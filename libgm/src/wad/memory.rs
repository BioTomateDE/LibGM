// SPDX-License-Identifier: GPL-3.0-only
use std::collections::HashMap;
use std::hash::BuildHasher;
use std::hash::Hash;

use crate::prelude::GMListChunk;
use crate::util::bench::Stopwatch;
use crate::util::fmt::format_bytes;
use crate::wad::GMData;

impl GMData {
    /// Tries to reduce memory footprint by shrinking `Vec`s and `HashMap`s so
    /// they don't take up unneeded space.
    ///
    /// This function may be useful to call once after data deserialization in a
    /// long-lived application (such as a GUI/TUI where the [`GMData`] is
    /// stored indefinitely). It is also useful to call this function after
    /// changing formats of lots of texture pages.
    ///
    /// You should probably not call this function frequently, as it consumes
    /// CPU power and will not meaningfully shrink your memory footprint by
    /// much if called repeatedly.
    ///
    /// Note: This function logs a message at the end.
    /// If you do not want this, use [`GMData::optimize_memory_silent`] instead.
    pub fn optimize_memory(&mut self) {
        let stopwatch = Stopwatch::start();
        let freed_bytes: usize = optimize_memory(self);
        let human_size: String = format_bytes(freed_bytes);
        log::debug!("Freed {human_size} ({freed_bytes} bytes) in {stopwatch}");
    }

    /// Tries to reduce memory footprint by shrinking `Vec`s and `HashMap`s so
    /// they don't take up unneeded space.
    ///
    /// This is [`GMData::optimize_memory`] except it does not log anything.
    pub fn optimize_memory_silent(&mut self) {
        optimize_memory(self);
    }
}

fn optimize_memory(data: &mut GMData) -> usize {
    let mut freed_bytes: usize = 0;

    // first, do all list chunks
    freed_bytes += shrink_vec(&mut data.animation_curves.animation_curves);
    freed_bytes += shrink_vec(&mut data.audio_groups.audio_groups);
    freed_bytes += shrink_vec(&mut data.audios.audios);
    freed_bytes += shrink_vec(&mut data.backgrounds.backgrounds);
    freed_bytes += shrink_vec(&mut data.codes.codes);
    freed_bytes += shrink_vec(&mut data.embedded_images.embedded_images);
    freed_bytes += shrink_vec(&mut data.extensions.extensions);
    freed_bytes += shrink_vec(&mut data.feature_flags.feature_flags);
    freed_bytes += shrink_vec(&mut data.filter_effects.filter_effects);
    freed_bytes += shrink_vec(&mut data.fonts.fonts);
    freed_bytes += shrink_vec(&mut data.functions.functions);
    freed_bytes += shrink_vec(&mut data.game_end_scripts.game_end_scripts);
    freed_bytes += shrink_vec(&mut data.game_objects.game_objects);
    freed_bytes += shrink_vec(&mut data.global_init_scripts.global_init_scripts);
    freed_bytes += shrink_vec(&mut data.particle_emitters.emitters);
    freed_bytes += shrink_vec(&mut data.particle_systems.particle_systems);
    freed_bytes += shrink_vec(&mut data.paths.paths);
    freed_bytes += shrink_vec(&mut data.rooms.rooms);
    freed_bytes += shrink_vec(&mut data.ui_nodes.ui_nodes);
    freed_bytes += shrink_vec(&mut data.scripts.scripts);
    freed_bytes += shrink_vec(&mut data.sequences.sequences);
    freed_bytes += shrink_vec(&mut data.shaders.shaders);
    freed_bytes += shrink_vec(&mut data.sounds.sounds);
    freed_bytes += shrink_vec(&mut data.sprites.sprites);
    freed_bytes += shrink_vec(&mut data.strings.strings);
    freed_bytes += shrink_vec(&mut data.texture_group_infos.texture_group_infos);
    freed_bytes += shrink_vec(&mut data.texture_page_items.texture_page_items);
    freed_bytes += shrink_vec(&mut data.texture_pages.texture_pages);
    freed_bytes += shrink_vec(&mut data.timelines.timelines);
    freed_bytes += shrink_vec(&mut data.variables.variables);

    // instructions don't have a known count before deserialization
    for code in data.codes.elements_mut() {
        freed_bytes += shrink_vec(&mut code.instructions);
    }

    // hashmaps suck
    for sequence in data.sequences.elements_mut() {
        freed_bytes += shrink_hashmap(&mut sequence.function_ids);
    }

    // internally stored vecs might have been overallocated
    for texture_page in data.texture_pages.elements_mut() {
        let Some(image) = &mut texture_page.image else {
            continue;
        };
        freed_bytes += image.optimize_memory();
    }

    freed_bytes
}

fn shrink_vec<T>(vector: &mut Vec<T>) -> usize {
    let before = vector.capacity();
    vector.shrink_to_fit();
    let after = vector.capacity();
    before - after
}

fn shrink_hashmap<K: Hash + Eq, V, S: BuildHasher>(hashmap: &mut HashMap<K, V, S>) -> usize {
    let before = hashmap.capacity();
    hashmap.shrink_to_fit();
    let after = hashmap.capacity();
    before - after
}
