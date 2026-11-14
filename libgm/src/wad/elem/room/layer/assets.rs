// SPDX-License-Identifier: GPL-3.0-only
mod particle_system;
mod sequence;
mod sprite;
mod text_item;

pub use self::particle_system::ParticleSystemInstance;
pub use self::sequence::SequenceInstance;
pub use self::sprite::SpriteInstance;
pub use self::text_item::TextItemInstance;
use crate::prelude::*;
use crate::wad::GMVersion;
use crate::wad::elem::GMElement;
use crate::wad::elem::room::tile::RoomTile;
use crate::wad::load::reader::DataReader;
use crate::wad::save::writer::DataWriter;

#[derive(Debug, Clone, PartialEq)]
pub struct Assets {
    pub legacy_tiles: Vec<RoomTile>,
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

        if reader.version >= GMVersion::GMS2_3 {
            sequences_pointer = reader.read_u32()?;
            if reader.version < GMVersion::GMS2_3_2 {
                nine_slices_pointer = reader.read_u32()?;
            }
            if reader.version >= GMVersion::GM2023_2 {
                particle_systems_pointer = reader.read_u32()?;
            }
            if reader.version >= GMVersion::GM2024_6 {
                text_items_pointer = reader.read_u32()?;
            }
        }

        reader.assert_pos(legacy_tiles_pointer, "Legacy Tiles")?;
        let legacy_tiles: Vec<RoomTile> = reader.read_pointer_list()?;

        reader.assert_pos(sprites_pointer, "Sprite Instances")?;
        let sprites: Vec<SpriteInstance> = reader.read_pointer_list()?;

        let mut sequences: Vec<SequenceInstance> = Vec::new();
        let mut nine_slices: Vec<SpriteInstance> = Vec::new();
        let mut particle_systems: Vec<ParticleSystemInstance> = Vec::new();
        let mut text_items: Vec<TextItemInstance> = Vec::new();

        if reader.version >= GMVersion::GMS2_3 {
            reader.assert_pos(sequences_pointer, "Sequences")?;
            sequences = reader.read_pointer_list()?;

            if reader.version < GMVersion::GMS2_3_2 {
                reader.assert_pos(nine_slices_pointer, "Nine Slices")?;
                nine_slices = reader.read_pointer_list()?;
            }
            if reader.version >= GMVersion::GM2023_2 {
                reader.assert_pos(particle_systems_pointer, "Room Particle Systems")?;
                particle_systems = reader.read_pointer_list()?;
            }

            if reader.version >= GMVersion::GM2024_6 {
                reader.assert_pos(text_items_pointer, "Text Items")?;
                text_items = reader.read_pointer_list()?;
            }
        }

        Ok(Self { legacy_tiles, sprites, sequences, nine_slices, particle_systems, text_items })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_pointer(&self.legacy_tiles);
        writer.write_pointer(&self.sprites);

        if writer.version() >= GMVersion::GMS2_3 {
            writer.write_pointer(&self.sequences);
            if writer.version() < GMVersion::GMS2_3_2 {
                writer.write_pointer(&self.nine_slices);
            }
            if writer.version() >= GMVersion::GM2023_2 {
                writer.write_pointer(&self.particle_systems);
            }
            if writer.version() >= GMVersion::GM2024_6 {
                writer.write_pointer(&self.text_items);
            }
        }
        writer.resolve_pointer(&self.legacy_tiles)?;
        writer.write_pointer_list(&self.legacy_tiles)?;

        writer.resolve_pointer(&self.sprites)?;
        writer.write_pointer_list(&self.sprites)?;

        if writer.version() >= GMVersion::GMS2_3 {
            writer.resolve_pointer(&self.sequences)?;
            writer.write_pointer_list(&self.sequences)?;

            if writer.version() < GMVersion::GMS2_3_2 {
                writer.resolve_pointer(&self.nine_slices)?;
                writer.write_pointer_list(&self.nine_slices)?;
            }
            if writer.version() >= GMVersion::GM2023_2 {
                writer.resolve_pointer(&self.particle_systems)?;
                writer.write_pointer_list(&self.particle_systems)?;
            }
            if writer.version() >= GMVersion::GM2024_6 {
                writer.resolve_pointer(&self.text_items)?;
                writer.write_pointer_list(&self.text_items)?;
            }
        }
        Ok(())
    }
}
