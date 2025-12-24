mod particle_system;
mod sequence;
mod sprite;
mod text_item;

pub use particle_system::ParticleSystemInstance;
pub use sequence::SequenceInstance;
pub use sprite::SpriteInstance;
pub use text_item::TextItemInstance;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMElement, room::tile::Tile},
        version::LTSBranch,
        serialize::builder::DataBuilder,
    },
    prelude::*,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Assets {
    pub legacy_tiles: Vec<Tile>,
    pub sprites: Vec<SpriteInstance>,
    pub sequences: Vec<SequenceInstance>,
    pub nine_slices: Vec<SpriteInstance>,
    pub particle_systems: Vec<ParticleSystemInstance>,
    pub text_items: Vec<TextItemInstance>,
}

impl GMElement for Assets {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let legacy_tiles_pointer = reader.read_u32()?;
        let sprites_pointer = reader.read_u32()?;
        let mut sequences_pointer: u32 = 0;
        let mut nine_slices_pointer: u32 = 0;
        let mut particle_systems_pointer: u32 = 0;
        let mut text_items_pointer: u32 = 0;

        if reader.general_info.is_version_at_least((2, 3)) {
            sequences_pointer = reader.read_u32()?;
            if !reader.general_info.is_version_at_least((2, 3, 2)) {
                nine_slices_pointer = reader.read_u32()?;
            }
            if reader
                .general_info
                .is_version_at_least((2023, 2, LTSBranch::PostLTS))
            {
                particle_systems_pointer = reader.read_u32()?;
            }
            if reader.general_info.is_version_at_least((2024, 6)) {
                text_items_pointer = reader.read_u32()?;
            }
        }

        reader.assert_pos(legacy_tiles_pointer, "Legacy Tiles")?;
        let legacy_tiles: Vec<Tile> = reader.read_pointer_list()?;

        reader.assert_pos(sprites_pointer, "Sprite Instances")?;
        let sprites: Vec<SpriteInstance> = reader.read_pointer_list()?;

        let mut sequences: Vec<SequenceInstance> = Vec::new();
        let mut nine_slices: Vec<SpriteInstance> = Vec::new();
        let mut particle_systems: Vec<ParticleSystemInstance> = Vec::new();
        let mut text_items: Vec<TextItemInstance> = Vec::new();

        if reader.general_info.is_version_at_least((2, 3)) {
            reader.assert_pos(sequences_pointer, "Sequences")?;
            sequences = reader.read_pointer_list()?;

            if !reader.general_info.is_version_at_least((2, 3, 2)) {
                reader.assert_pos(nine_slices_pointer, "Nine Slices")?;
                nine_slices = reader.read_pointer_list()?;
            }
            if reader
                .general_info
                .is_version_at_least((2023, 2, LTSBranch::PostLTS))
            {
                reader.assert_pos(particle_systems_pointer, "Particle Systems")?;
                particle_systems = reader.read_pointer_list()?;
            }

            if reader.general_info.is_version_at_least((2024, 6)) {
                reader.assert_pos(text_items_pointer, "Text Items")?;
                text_items = reader.read_pointer_list()?;
            }
        }

        Ok(Self {
            legacy_tiles,
            sprites,
            sequences,
            nine_slices,
            particle_systems,
            text_items,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_pointer(&self.legacy_tiles);
        builder.write_pointer(&self.sprites);

        if builder.is_gm_version_at_least((2, 3)) {
            builder.write_pointer(&self.sequences);
            if !builder.is_gm_version_at_least((2, 3, 2)) {
                builder.write_pointer(&self.nine_slices);
            }
            if builder.is_gm_version_at_least((2023, 2, LTSBranch::PostLTS)) {
                builder.write_pointer(&self.particle_systems);
            }
            if builder.is_gm_version_at_least((2024, 6)) {
                builder.write_pointer(&self.text_items);
            }
        }
        builder.resolve_pointer(&self.legacy_tiles)?;
        builder.write_pointer_list(&self.legacy_tiles)?;

        builder.resolve_pointer(&self.sprites)?;
        builder.write_pointer_list(&self.sprites)?;

        if builder.is_gm_version_at_least((2, 3)) {
            builder.resolve_pointer(&self.sequences)?;
            builder.write_pointer_list(&self.sequences)?;

            if !builder.is_gm_version_at_least((2, 3, 2)) {
                builder.resolve_pointer(&self.nine_slices)?;
                builder.write_pointer_list(&self.nine_slices)?;
            }
            if builder.is_gm_version_at_least((2023, 2, LTSBranch::PostLTS)) {
                builder.resolve_pointer(&self.particle_systems)?;
                builder.write_pointer_list(&self.particle_systems)?;
            }
            if builder.is_gm_version_at_least((2024, 6)) {
                builder.resolve_pointer(&self.text_items)?;
                builder.write_pointer_list(&self.text_items)?;
            }
        }
        Ok(())
    }
}
