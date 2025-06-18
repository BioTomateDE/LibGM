use serde::{Deserialize, Serialize};
use crate::deserialize::backgrounds::GMBackgroundGMS2Data;
use crate::export_mod::export::{edit_field, edit_field_convert, edit_field_convert_option, ModExporter, ModRef};
use crate::export_mod::unordered_list::{export_changes_unordered_list, EditUnorderedList};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddBackground {
    pub name: ModRef,
    pub transparent: bool,
    pub smooth: bool,
    pub preload: bool,
    pub texture: Option<ModRef>,    // TexturePageItem ref
    pub gms2_data: Option<ModBackgroundGMS2Data>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditBackground {
    pub name: Option<ModRef>,
    pub transparent: Option<bool>,
    pub smooth: Option<bool>,
    pub preload: Option<bool>,
    pub texture: Option<Option<ModRef>>,    // TexturePageItem ref
    pub gms2_data: Option<ModBackgroundGMS2Data>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModBackgroundGMS2Data {
    pub tile_width: u32,
    pub tile_height: u32,
    pub output_border_x: u32,
    pub output_border_y: u32,
    pub tile_columns: u32,
    pub items_per_tile_count: usize,
    pub frame_length: i64,
    pub tile_ids: Vec<u32>,
}


impl ModExporter<'_, '_> {
    pub fn export_backgrounds(&self) -> Result<EditUnorderedList<AddBackground, EditBackground>, String> {
        export_changes_unordered_list(
            &self.original_data.backgrounds.backgrounds,
            &self.modified_data.backgrounds.backgrounds,
            |i| Ok(AddBackground {
                name: self.convert_string_ref(&i.name)?,
                transparent: i.transparent,
                smooth: i.smooth,
                preload: i.preload,
                texture: self.convert_texture_ref_opt(&i.texture)?,
                gms2_data: i.gms2_data.as_ref().map(convert_gms2_data),
            }),
            |o, m| Ok(EditBackground {
                name: edit_field_convert(&o.name, &m.name, |r| self.convert_string_ref(r))?,
                transparent: edit_field(&o.transparent, &m.transparent),
                smooth: edit_field(&o.smooth, &m.smooth),
                preload: edit_field(&o.preload, &m.preload),
                texture: edit_field_convert_option(&o.texture, &m.texture, |r| self.convert_texture_ref(r))?,
                gms2_data: edit_field_convert_option(&o.gms2_data, &m.gms2_data, |i| Ok(convert_gms2_data(i)))?.unwrap_or(None),
            }),
            false,
        )
    }
}


/// truly extremely useful
fn convert_gms2_data(i: &GMBackgroundGMS2Data) -> ModBackgroundGMS2Data {
    ModBackgroundGMS2Data {
        tile_width: i.tile_width,
        tile_height: i.tile_height,
        output_border_x: i.output_border_x,
        output_border_y: i.output_border_y,
        tile_columns: i.tile_columns,
        items_per_tile_count: i.items_per_tile_count,
        frame_length: i.frame_length,
        tile_ids: i.tile_ids.clone(),
    }
}

