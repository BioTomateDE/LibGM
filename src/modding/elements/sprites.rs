use serde::{Deserialize, Serialize};
use crate::gamemaker::elements::sequence::GMAnimSpeedType;
use crate::gamemaker::elements::sprites::{GMSpriteMaskEntry, GMSpriteNineSlice, GMSpriteNineSliceTileMode, GMSpriteSepMaskType, GMSpriteSpecial, GMSpriteType};
use crate::modding::export::{edit_field, edit_field_convert, edit_field_convert_option, wrap_edit_option, EditWrapper, ModExporter, ModRef};
use crate::modding::ordered_list::{export_changes_ordered_list, DataChange};
use crate::modding::elements::sequences::AddSequence;
use crate::modding::unordered_list::{export_changes_unordered_list, EditUnorderedList};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSprite {
    pub name: ModRef,
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
    pub sep_masks: GMSpriteSepMaskType,
    pub origin_x: i32,
    pub origin_y: i32,
    pub textures: Vec<ModRef>,
    pub collision_masks: Vec<GMSpriteMaskEntry>,
    pub special_fields: Option<AddSpriteSpecial>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSpriteSpecial {
    /// Version of Special Thingy
    pub special_version: u32,
    // pub sprite_type: AddSpriteType,
    /// GMS 2
    pub playback_speed: f32,
    /// GMS 2
    pub playback_speed_type: GMAnimSpeedType,
    /// Special Version 2
    pub sequence: Option<AddSequence>,
    /// Special Version 3
    pub nine_slice: Option<ModSpriteNineSlice>,
    // /// SWF
    // pub yyswf: Option<GMSpriteTypeSWF>,   // just reuse the gm struct; im not touching yyswf with a 3m pole
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditSprite {
    pub name: Option<ModRef>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub margin_left: Option<i32>,
    pub margin_right: Option<i32>,
    pub margin_bottom: Option<i32>,
    pub margin_top: Option<i32>,
    pub transparent: Option<bool>,
    pub smooth: Option<bool>,
    pub preload: Option<bool>,
    pub bbox_mode: Option<i32>,
    pub sep_masks: Option<GMSpriteSepMaskType>,
    pub origin_x: Option<i32>,
    pub origin_y: Option<i32>,
    pub textures: Vec<DataChange<ModRef>>,
    pub collision_masks: EditUnorderedList<GMSpriteMaskEntry, GMSpriteMaskEntry>,
    pub special_fields: Option<EditWrapper<AddSpriteSpecial, EditSpriteSpecial>>,
}
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditSpriteSpecial {
    pub special_version: Option<u32>,
    // pub sprite_type: Option<GMSpriteType>,
    pub playback_speed: Option<f32>,
    pub playback_speed_type: Option<GMAnimSpeedType>,
    pub sequence: Option<Option<AddSequence>>,
    pub nine_slice: Option<ModSpriteNineSlice>,
    // pub yyswf: Option<Option<GMSpriteTypeSWF>>,   // just reuse the gm struct; im not touching yyswf with a 3m pole
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModSpriteNineSlice {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub enabled: bool,
    pub tile_modes: [GMSpriteNineSliceTileMode; 5],
}


impl ModExporter<'_, '_> {
    pub fn export_sprites(&self) -> Result<EditUnorderedList<AddSprite, EditSprite>, String> {
        export_changes_unordered_list(
            &self.original_data.sprites.sprites,
            &self.modified_data.sprites.sprites,
            |i| Ok(AddSprite {
                name: self.convert_string_ref(&i.name)?,
                width: i.width,
                height: i.height,
                margin_left: i.margin_left,
                margin_right: i.margin_right,
                margin_bottom: i.margin_bottom,
                margin_top: i.margin_top,
                transparent: i.transparent,
                smooth: i.smooth,
                preload: i.preload,
                bbox_mode: i.bbox_mode,
                sep_masks: i.sep_masks,
                origin_x: i.origin_x,
                origin_y: i.origin_y,
                textures: i.textures.iter().flatten().map(|r| self.convert_texture_ref(r)).collect::<Result<Vec<_>, String>>()?,
                collision_masks: i.collision_masks.clone(),
                special_fields: i.special_fields.as_ref().map(|i| self.add_specials(i)).transpose()?,
            }),
            |o, m| Ok(EditSprite {
                name: edit_field_convert(&o.name, &m.name, |r| self.convert_string_ref(r))?,
                width: edit_field(&o.width, &m.width),
                height: edit_field(&o.height, &m.height),
                margin_left: edit_field(&o.margin_left, &m.margin_left),
                margin_right: edit_field(&o.margin_right, &m.margin_right),
                margin_bottom: edit_field(&o.margin_bottom, &m.margin_bottom),
                margin_top: edit_field(&o.margin_top, &m.margin_top),
                transparent: edit_field(&o.transparent, &m.transparent),
                smooth: edit_field(&o.smooth, &m.smooth),
                preload: edit_field(&o.preload, &m.preload),
                bbox_mode: edit_field(&o.bbox_mode, &m.bbox_mode),
                sep_masks: edit_field(&o.sep_masks, &m.sep_masks),
                origin_x: edit_field(&o.origin_x, &m.origin_x),
                origin_y: edit_field(&o.origin_y, &m.origin_y),
                textures: export_changes_ordered_list(
                    &o.textures.iter().flatten().collect(),   // remove null entries
                    &m.textures.iter().flatten().collect(),
                    |r| self.convert_texture_ref(r)
                )?,
                collision_masks: export_changes_unordered_list(
                    &o.collision_masks,
                    &m.collision_masks,
                    |i| Ok(i.clone()),
                    |_, m| Ok(m.clone()),
                    false,
                )?,
                special_fields: wrap_edit_option(
                    &o.special_fields,
                    &m.special_fields,
                    |i| self.add_specials(i),
                    |o, m| self.edit_specials(o, m),
                )?,
            }),
            false,
        )
    }
    
    fn add_specials(&self, i: &GMSpriteSpecial) -> Result<AddSpriteSpecial, String> {
        if !matches!(i.sprite_type, GMSpriteType::Normal(_)) {
            return Err("Spine and SWF sprites are not supported yet for modding".to_string())   // TODO
        }
        Ok(AddSpriteSpecial {
            special_version: i.special_version,
            // sprite_type: i.sprite_type.clone(),
            playback_speed: i.playback_speed,
            playback_speed_type: i.playback_speed_type,
            sequence: i.sequence.as_ref().map(|i| self.add_sequence(&i)).transpose()?,
            nine_slice: i.nine_slice.as_ref().map(|i| ModSpriteNineSlice {
                left: i.left,
                top: i.top,
                right: i.right,
                bottom: i.bottom,
                enabled: i.enabled,
                tile_modes: i.tile_modes,
            })
            // yyswf: i.yyswf.clone(),
        })
    }
    
    fn edit_specials(&self, o: &GMSpriteSpecial, m: &GMSpriteSpecial) -> Result<EditSpriteSpecial, String> {
        Ok(EditSpriteSpecial {
            special_version: edit_field(&o.special_version, &m.special_version),
            // sprite_type: edit_field(&o.sprite_type, &m.sprite_type),
            playback_speed: edit_field(&o.playback_speed, &m.playback_speed),
            playback_speed_type: edit_field(&o.playback_speed_type, &m.playback_speed_type),
            sequence: edit_field_convert_option(&o.sequence, &m.sequence, |i| self.add_sequence(i))?,
            nine_slice: convert_nine_slice(&m.nine_slice)
            // yyswf: edit_field(&o.yyswf, &m.yyswf),
        })
    }
}

fn convert_nine_slice(i: &Option<GMSpriteNineSlice>) -> Option<ModSpriteNineSlice> {
    i.as_ref().map(|i| ModSpriteNineSlice {
        left: i.left,
        top: i.top,
        right: i.right,
        bottom: i.bottom,
        enabled: i.enabled,
        tile_modes: i.tile_modes,
    })
}

