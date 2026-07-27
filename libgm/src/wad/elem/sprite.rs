// SPDX-License-Identifier: GPL-3.0-only
pub mod nine_slice;
pub mod spine;
pub mod swf;

pub use nine_slice::NineSlice;

use crate::gm_enum::gm_enum;
use crate::prelude::*;
use crate::util::assert;
use crate::util::init::vec_with_capacity;
use crate::wad::Blob;
use crate::wad::GMVersion;
use crate::wad::chunk::gm_named_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::elem::sequence::Sequence;
use crate::wad::elem::sequence::SpeedType;
use crate::wad::elem::texture_page_item::TexturePageItem;
use crate::wad::load::reader::DataReader;
use crate::wad::reference::GMRef;
use crate::wad::save::writer::DataWriter;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Sprites {
    pub elems: Vec<Option<Sprite>>,
}

gm_named_list_chunk!(SPRT, Sprites, Sprite, nullable);

impl GMElement for Sprites {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let elems: Vec<Option<Sprite>> = reader.read_pointer_list_opt()?;
        Ok(Self { elems })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_pointer_list_opt(&self.elems)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sprite {
    pub name: GMRef<String>,
    pub width: u32,
    pub height: u32,
    pub margin_left: i32,
    pub margin_right: i32,
    pub margin_bottom: i32,
    pub margin_top: i32,
    pub transparent: bool,
    pub smooth: bool,
    pub preload: bool,
    pub bbox_mode: i32,
    pub sep_masks: SepMaskType,
    pub origin_x: i32,
    pub origin_y: i32,
    pub textures: Vec<GMRef<TexturePageItem>>,
    pub collision_masks: Vec<MaskEntry>,
    pub special_fields: Option<Special>,
}

#[allow(clippy::too_many_lines)] // TODO: these functions fucking suck
impl GMElement for Sprite {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let width = reader.read_u32()?;
        let height = reader.read_u32()?;
        let margin_left = reader.read_i32()?;
        let margin_right = reader.read_i32()?;
        let margin_bottom = reader.read_i32()?;
        let margin_top = reader.read_i32()?;
        let transparent = reader.read_bool32()?;
        let smooth = reader.read_bool32()?;
        let preload = reader.read_bool32()?;
        let bbox_mode = reader.read_i32()?;
        let sep_masks: SepMaskType = reader.read_enum()?;
        let origin_x = reader.read_i32()?;
        let origin_y = reader.read_i32()?;
        let mut textures: Vec<GMRef<TexturePageItem>> = Vec::new();
        let mut collision_masks: Vec<MaskEntry> = Vec::new();
        let mut special_fields: Option<Special> = None;

        // Combination of these conditions may be incorrect
        if reader.read_i32()? == -1 && reader.version >= GMVersion::GMS2 {
            let special_version = reader.read_u32()?;
            let special_sprite_type = reader.read_u32()?;

            let mut sequence: Option<Sequence> = None;
            let mut nine_slice: Option<NineSlice> = None;
            let swf: Option<swf::Data> = None;

            let playback_speed = reader.read_f32()?;
            let playback_speed_type: SpeedType = reader.read_enum()?;
            // both of these seem to be not an offset but instead a position (see
            // UndertaleModLib/Models/UndertaleSprite.cs:507)
            let sequence_pos = if special_version >= 2 {
                reader.read_u32()?
            } else {
                0
            };
            let nine_slice_pos = if special_version >= 3 {
                reader.read_u32()?
            } else {
                0
            };

            let special_data: SpecialData = match &special_sprite_type {
                0 => {
                    // Normal
                    textures = Self::read_texture_list(reader)?;
                    // Read mask data
                    let mut mask_width = width;
                    let mut mask_height = height;
                    if reader.version >= GMVersion::GM2024_6 {
                        mask_width = (margin_right - margin_left + 1) as u32;
                        mask_height = (margin_bottom - margin_top + 1) as u32;
                    }
                    collision_masks = read_mask_data(reader, mask_width, mask_height)
                        .ctx("parsing mask data for normal special Sprite")?;
                    SpecialData::Normal
                }

                1 => {
                    // SWF
                    // [From UndertaleModTool] "This code does not work all the time for some
                    // reason"
                    let swf_version = reader.read_i32()?;
                    if swf_version != 7 && swf_version != 8 {
                        bail!("Invalid SWF version {swf_version} for Sprite {name:?}");
                    }
                    if swf_version == 8 {
                        textures = Self::read_texture_list(reader)?;
                    }

                    // Read YYSWF
                    reader.align(4)?;
                    let jpeg_len = (reader.read_i32()? & i32::MAX) as u32;
                    let yyswf_version = reader.read_i32()?;
                    if !matches!(yyswf_version, 7 | 8) {
                        bail!("Expected YYSWF Version 7 or 8 but got {yyswf_version}");
                    }
                    let jpeg_table: Vec<u8> = reader
                        .read_bytes_dyn(jpeg_len)
                        .ctx("reading YYSWF JPEG Table")?
                        .to_vec();
                    reader.align(4)?;
                    let timeline = swf::Timeline::deserialize(reader)?;
                    SpecialData::Swf(swf::Data {
                        swf_version,
                        yyswf_version,
                        jpeg_table: Blob(jpeg_table),
                        timeline,
                    })
                }

                2 => {
                    // Spine
                    reader.align(4)?;
                    if reader.version >= GMVersion::GM2023_1 {
                        textures = Self::read_texture_list(reader)?;
                    }

                    let spine_version = reader.read_i32()?;
                    if spine_version >= 3 {
                        reader.read_gms2_chunk_version("Spine Cache Version")?;
                    }

                    let json_length = reader.read_u32()?;
                    let atlas_length = reader.read_u32()?;
                    let mut spine_textures: Vec<spine::TextureEntry> = Vec::new();
                    let spine_json: String;
                    let spine_atlas: String;

                    // Version 1 - only one single PNG atlas.
                    // Version 2 - can be multiple atlases.
                    // Version 3 - an atlas can be a QOI blob.
                    match spine_version {
                        1 => {
                            let blob_size = reader.read_u32()?;
                            let page_width = reader.read_u32()?;
                            let page_height = reader.read_u32()?;

                            spine_json = spine::Data::read_weird_string(reader, json_length)?;
                            spine_atlas = spine::Data::read_weird_string(reader, atlas_length)?;
                            let texture_blob: Vec<u8> = reader
                                .read_bytes_dyn(blob_size)
                                .ctx("reading Spine v1 texture blob")?
                                .to_vec();

                            spine_textures.push(spine::TextureEntry {
                                page_width,
                                page_height,
                                data: spine::texture_entry::Data::Pre2023_1(Blob(texture_blob)),
                            });
                        }
                        2 | 3 => {
                            let texture_count = reader.read_u32()?;
                            spine_json = spine::Data::read_weird_string(reader, json_length)?;
                            spine_atlas = spine::Data::read_weird_string(reader, atlas_length)?;

                            spine_textures = vec_with_capacity(texture_count)?;
                            for _ in 0..texture_count {
                                spine_textures.push(
                                    spine::TextureEntry::deserialize(reader)
                                        .ctx("parsing Texture Entry for Spine v2/3 data")?,
                                );
                            }
                        }
                        _ => bail!(
                            "Expected Spine Version 1, 2 or 3 but got {spine_version} for Special \
                             Sprite"
                        ),
                    }

                    SpecialData::Spine(spine::Data {
                        version: spine_version,
                        textures: spine_textures,
                        json: spine_json,
                        atlas: spine_atlas,
                    })
                }
                3 => {
                    // TODO(weak): implement vector eventually
                    bail!(
                        "Vector Sprite Type not yet supported; will be implemented when UTMT \
                         stops using raw ints for this"
                    );
                }

                other => {
                    bail!("Invalid Sprite Type {other} for Sprite {name:?}")
                }
            };

            if sequence_pos != 0 {
                reader.assert_pos(sequence_pos, "Sequence")?;
                let ctx = || format!("parsing Sequence for Sprite {name:?}");
                reader.read_gms2_chunk_version("Sequence").ctx(ctx)?;
                sequence = Some(Sequence::deserialize(reader).ctx(ctx)?);
            }

            if nine_slice_pos != 0 {
                reader.assert_pos(nine_slice_pos, "Nine Slice")?;
                let ctx = || format!("parsing Nine Slice for Sprite {name:?}");
                nine_slice = Some(NineSlice::deserialize(reader).ctx(ctx)?);
            }

            special_fields = Some(Special {
                special_version,
                data: special_data,
                playback_speed,
                playback_speed_type,
                sequence,
                nine_slice,
                swf,
            });
        } else {
            // Normal sprite
            reader.cur_pos -= 4; // Unread the not -1
            // Read into `textures`
            textures = Self::read_texture_list(reader)?;
            // Read mask data
            let mut mask_width = width;
            let mut mask_height = height;
            if reader.version >= GMVersion::GM2024_6 {
                mask_width = (margin_right - margin_left + 1) as u32;
                mask_height = (margin_bottom - margin_top + 1) as u32;
            }
            collision_masks = read_mask_data(reader, mask_width, mask_height)
                .ctx("reading mask data for normal Sprite")?;
        }

        Ok(Self {
            name,
            width,
            height,
            margin_left,
            margin_right,
            margin_bottom,
            margin_top,
            transparent,
            smooth,
            preload,
            bbox_mode,
            sep_masks,
            origin_x,
            origin_y,
            textures,
            collision_masks,
            special_fields,
        })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_gm_string(self.name)?;
        writer.write_u32(self.width);
        writer.write_u32(self.height);
        writer.write_i32(self.margin_left);
        writer.write_i32(self.margin_right);
        writer.write_i32(self.margin_bottom);
        writer.write_i32(self.margin_top);
        writer.write_bool32(self.transparent);
        writer.write_bool32(self.smooth);
        writer.write_bool32(self.preload);
        writer.write_i32(self.bbox_mode);
        writer.write_enum(self.sep_masks);
        writer.write_i32(self.origin_x);
        writer.write_i32(self.origin_y);
        if self.special_fields.is_none() {
            Self::build_texture_list(writer, &self.textures)?;
            self.build_mask_data(writer, &self.collision_masks)?;
            return Ok(());
        }

        let special_fields: &Special = self.special_fields.as_ref().unwrap();
        writer.write_i32(-1);
        writer.write_u32(special_fields.special_version);
        writer.write_u32(match special_fields.data {
            SpecialData::Normal => 0,
            SpecialData::Swf(_) => 1,
            SpecialData::Spine(_) => 2,
        });

        if writer.version() >= GMVersion::GMS2 {
            writer.write_f32(special_fields.playback_speed);
            writer.write_enum(special_fields.playback_speed_type);
            if special_fields.special_version >= 2 {
                if special_fields.sequence.is_some() {
                    writer.write_pointer(&special_fields.sequence);
                } else {
                    writer.write_u32(0);
                }
            }
            if special_fields.special_version >= 3 {
                if special_fields.nine_slice.is_some() {
                    writer.write_pointer(&special_fields.nine_slice);
                } else {
                    writer.write_u32(0);
                }
            }
        }

        match &special_fields.data {
            SpecialData::Normal => {
                Self::build_texture_list(writer, &self.textures)?;
                self.build_mask_data(writer, &self.collision_masks)?;
            }
            SpecialData::Swf(swf) => {
                writer.write_i32(swf.swf_version);
                if swf.swf_version == 8 {
                    Self::build_texture_list(writer, &self.textures)?;
                }
                writer.align(4);
                writer.write_usize(swf.jpeg_table.len())?; // Can be unset?
                writer.write_i32(swf.yyswf_version);
                writer.write_bytes(&swf.jpeg_table);
                writer.align(4);
                writer.write_elem(&swf.timeline)?;
            }
            SpecialData::Spine(spine) => {
                writer.align(4);
                let json_blob: Vec<u8> = spine::Data::build_weird_string(&spine.json);
                let atlas_blob: Vec<u8> = spine::Data::build_weird_string(&spine.atlas);
                if writer.version() >= GMVersion::GM2023_1 {
                    writer.write_simple_list(&spine.textures)?;
                }
                writer.write_i32(spine.version);
                if spine.version >= 3 {
                    writer.write_i32(1); // Spine cache version 1
                }
                writer.write_usize(json_blob.len())?;
                writer.write_usize(atlas_blob.len())?;

                match spine.version {
                    1 => {
                        let atlas: &spine::TextureEntry = spine
                            .textures
                            .first()
                            .ok_or("Spine Sprite's texture list empty in Spine Version 1")?;
                        let spine::texture_entry::Data::Pre2023_1(ref texture_blob) = atlas.data
                        else {
                            bail!(
                                "Expected Pre2023_1 texture data in Sprite Spine Version 1 but \
                                 got Post2023_1"
                            );
                        };
                        writer.write_usize(texture_blob.len())?;
                        writer.write_u32(atlas.page_width);
                        writer.write_u32(atlas.page_height);
                        writer.write_bytes(&json_blob);
                        writer.write_bytes(&atlas_blob);
                        writer.write_bytes(texture_blob);
                    }
                    2 | 3 => {
                        writer.write_usize(spine.textures.len())?;
                        writer.write_bytes(&json_blob);
                        writer.write_bytes(&atlas_blob);
                        for texture_entry in &spine.textures {
                            writer.write_u32(texture_entry.page_width);
                            writer.write_u32(texture_entry.page_height);
                            if writer.version() >= GMVersion::GM2023_1 {
                                if let spine::texture_entry::Data::Post2023_1(length) =
                                    texture_entry.data
                                {
                                    writer.write_u32(length);
                                } else {
                                    bail!(
                                        "Expected Post2023_1 Sprite Spine texture data in 2023.1+"
                                    );
                                }
                            } else if let spine::texture_entry::Data::Pre2023_1(texture_blob) =
                                &texture_entry.data
                            {
                                writer.write_usize(texture_blob.len())?;
                                writer.write_bytes(texture_blob);
                            } else {
                                bail!("Expected Pre2023_1 Sprite Spine texture data in pre 2023.1");
                            }
                        }
                    }
                    other => bail!("Invalid Sprite Spine Version {other}; should be 1, 2 or 3"),
                }
            }
        }

        if writer.version() >= GMVersion::GMS2 {
            if special_fields.special_version >= 2
                && matches!(special_fields.data, SpecialData::Normal)
                && let Some(ref sequence) = special_fields.sequence
            {
                writer.resolve_pointer(&special_fields.sequence)?;
                writer.write_u32(1); // SEQN version
                writer.write_elem(sequence)?;
            }
            if special_fields.special_version >= 3
                && let Some(ref nine_slice) = special_fields.nine_slice
            {
                writer.resolve_pointer(&special_fields.nine_slice)?;
                writer.write_elem(nine_slice)?;
            }
        }
        Ok(())
    }

    fn deserialize_pre_padding(reader: &mut DataReader) -> Result<()> {
        reader.align(4)?;
        Ok(())
    }

    fn serialize_pre_padding(&self, writer: &mut DataWriter) -> Result<()> {
        writer.align(4);
        Ok(())
    }
}

impl Sprite {
    fn read_texture_list(reader: &mut DataReader) -> Result<Vec<GMRef<TexturePageItem>>> {
        let count = reader.read_count("Sprite texture")?;
        let ctx = || format!("reading {count} Sprite textures");
        let mut textures: Vec<GMRef<TexturePageItem>> = vec_with_capacity(count).ctx(ctx)?;
        for _ in 0..count {
            textures.push(reader.read_gm_texture().ctx(ctx)?);
        }
        Ok(textures)
    }

    fn build_texture_list(
        writer: &mut DataWriter,
        texture_list: &Vec<GMRef<TexturePageItem>>,
    ) -> Result<()> {
        writer.write_usize(texture_list.len())?;
        for &texture_page_item_ref in texture_list {
            writer.write_gm_texture(texture_page_item_ref)?;
        }
        Ok(())
    }

    fn build_mask_data(&self, writer: &mut DataWriter, masks: &Vec<MaskEntry>) -> Result<()> {
        let count = masks.len() as u32;
        writer.write_u32(count);

        let start = writer.pos();

        for mask in masks {
            writer.write_bytes(&mask.data);
        }

        writer.align(4);
        let written_bytes = writer.pos() - start;

        let (width, height) = if writer.version() >= GMVersion::GM2024_6 {
            (
                self.margin_right as u32 - self.margin_left as u32 + 1,
                self.margin_bottom as u32 - self.margin_top as u32 + 1,
            )
        } else {
            (self.width, self.height)
        };

        let rounded_width = width.next_multiple_of(8); // Align to 8
        let data_bits = rounded_width * height * count;
        let data_bits = data_bits.next_multiple_of(32); // Align to 32 bits
        let data_bytes = data_bits / 8;
        assert::int(written_bytes, data_bytes, "Sprite Mask Data Size")?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Special {
    /// Version of Special Thingy
    pub special_version: u32,
    pub data: SpecialData,
    /// GMS 2
    pub playback_speed: f32,
    /// GMS 2
    pub playback_speed_type: SpeedType,
    /// Special Version 2
    pub sequence: Option<Sequence>,
    /// Special Version 3
    pub nine_slice: Option<NineSlice>,
    /// YYSWF
    pub swf: Option<swf::Data>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpecialData {
    Normal,
    Swf(swf::Data),
    Spine(spine::Data),
}

gm_enum!(SepMaskType {
    AxisAlignedRect = 0,
    Precise = 1,
    RotatedRect = 2,
});

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaskEntry {
    pub data: Blob<Vec<u8>>,
    pub width: u32,
    pub height: u32,
}

fn read_mask_data(reader: &mut DataReader, width: u32, height: u32) -> Result<Vec<MaskEntry>> {
    let mask_count = reader.read_u32()?;
    let mut collision_masks: Vec<MaskEntry> = vec_with_capacity(mask_count)?;

    let length = width.div_ceil(8) * height;
    let start = reader.cur_pos;

    for _ in 0..mask_count {
        let data: Vec<u8> = reader
            .read_bytes_dyn(length)
            .ctx("reading Mask Data")?
            .to_vec();
        collision_masks.push(MaskEntry { data: Blob(data), width, height });
    }

    reader.align(4)?;

    let actual_size = reader.cur_pos - start;
    let expected_size = calculate_mask_data_size(width, height, mask_count);
    reader.assert_int(actual_size, expected_size, "Sprite Mask Data Size")?;

    Ok(collision_masks)
}

#[must_use]
const fn calculate_mask_data_size(width: u32, height: u32, mask_count: u32) -> u32 {
    let rounded_width = width.next_multiple_of(8); // Align to 8 bits
    let data_bits = rounded_width * height * mask_count;
    let data_bits = data_bits.next_multiple_of(32); // Align to 32 bits
    data_bits / 8
}
