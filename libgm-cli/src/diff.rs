use std::fmt::Debug;

use libgm::wad::GMData;
use similar_asserts::SimpleDiff;

pub fn print_diff<T: Debug + PartialEq + ?Sized>(old: &T, new: &T) {
    let old = format!("{old:#?}");
    let new = format!("{new:#?}");
    println!("{}", SimpleDiff::from_str(&old, &new, "Old", "New"));
}

pub fn print_diffs(old: &GMData, new: &GMData) {
    let mut same = true;

    macro_rules! diffs {
        ($($field:ident)*) => {{ $(
            if old.$field != new.$field {
                same = false;
                log::warn!(concat!("Difference for ", stringify!($field), ":"));
                $crate::diff::print_diff(&old.$field, &new.$field);
            }
        )* }};
    }

    diffs!(
         animation_curves
         audio_groups
         audios
         backgrounds
         codes
         embedded_images
         extensions
         feature_flags
         filter_effects
         fonts
         functions
         game_end_scripts
         game_objects
         general_info
         global_init_scripts
         language_info
         options
         particle_emitters
         particle_systems
         paths
         rooms
         ui_nodes
         scripts
         sequences
         shaders
         sounds
         sprites
         tags
         texture_group_infos
         texture_page_items
         texture_pages
         timelines
         variables
    );

    if same {
        println!("Data files are the same!");
    }
}
