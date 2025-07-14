use crate::modding::export::{ModExporter, ModRef};
use crate::gamemaker::deserialize::GMRef;
use crate::gamemaker::elements::audio_groups::GMAudioGroup;
use crate::gamemaker::elements::backgrounds::GMBackground;
use crate::gamemaker::elements::code::GMCode;
use crate::gamemaker::elements::embedded_audio::GMEmbeddedAudio;
use crate::gamemaker::elements::embedded_textures::GMEmbeddedTexture;
use crate::gamemaker::elements::functions::GMFunction;
use crate::gamemaker::elements::game_objects::GMGameObject;
use crate::gamemaker::elements::rooms::GMRoom;
use crate::gamemaker::elements::sprites::GMSprite;
use crate::gamemaker::elements::texture_page_items::GMTexturePageItem;
use crate::gamemaker::elements::variables::GMVariable;


impl ModExporter<'_, '_> {
    pub fn convert_audio_ref(&self, gm_audio_ref: &GMRef<GMEmbeddedAudio>) -> Result<ModRef, String> {
        convert_reference(gm_audio_ref, &self.original_data.audios.audios, &self.modified_data.audios.audios)
    }
    pub fn convert_audio_group_ref(&self, gm_audio_group_ref: &GMRef<GMAudioGroup>) -> Result<ModRef, String> {
        convert_reference(gm_audio_group_ref, &self.original_data.audio_groups.audio_groups, &self.modified_data.audio_groups.audio_groups)
    }
    pub fn convert_background_ref(&self, gm_background_ref: &GMRef<GMBackground>) -> Result<ModRef, String> {
        convert_reference(gm_background_ref, &self.original_data.backgrounds.backgrounds, &self.modified_data.backgrounds.backgrounds)
    }
    pub fn convert_code_ref(&self, gm_code_ref: &GMRef<GMCode>) -> Result<ModRef, String> {
        convert_reference(gm_code_ref, &self.original_data.codes.codes, &self.modified_data.codes.codes)
    }
    pub fn convert_function_ref(&self, gm_function_ref: &GMRef<GMFunction>) -> Result<ModRef, String> {
        convert_reference(gm_function_ref, &self.original_data.functions.functions, &self.modified_data.functions.functions)
    }
    // TODO continue
    pub fn convert_game_object_ref(&self, gm_game_object_ref: &GMRef<GMGameObject>) -> Result<ModRef, String> {
        convert_reference(gm_game_object_ref, &self.original_data.game_objects.game_objects, &self.modified_data.game_objects.game_objects)
    }
    pub fn convert_room_ref(&self, gm_room_ref: &GMRef<GMRoom>) -> Result<ModRef, String> {
        convert_reference(gm_room_ref, &self.original_data.rooms.rooms, &self.modified_data.rooms.rooms)
    }
    pub fn convert_sprite_ref(&self, gm_sprite_ref: &GMRef<GMSprite>) -> Result<ModRef, String> {
        convert_reference(gm_sprite_ref, &self.original_data.sprites.sprites, &self.modified_data.sprites.sprites)
    }
    pub fn convert_string_ref(&self, gm_string_ref: &GMRef<String>) -> Result<ModRef, String> {
        convert_reference(gm_string_ref, &self.original_data.strings.strings, &self.modified_data.strings.strings)
    }
    /// TODO make custom function for texture page items (since texture contents are not checked)
    pub fn convert_texture_ref(&self, gm_texture_ref: &GMRef<GMTexturePageItem>) -> Result<ModRef, String> {
        convert_reference(gm_texture_ref, &self.original_data.texture_page_items.texture_page_items, &self.modified_data.texture_page_items.texture_page_items)
    }
    pub fn convert_texture_page_ref(&self, gm_texture_page_ref: &GMRef<GMEmbeddedTexture>) -> Result<ModRef, String> {
        convert_reference(gm_texture_page_ref, &self.original_data.embedded_textures.texture_pages, &self.modified_data.embedded_textures.texture_pages)
    }
    pub fn convert_variable_ref(&self, gm_variable_ref: &GMRef<GMVariable>) -> Result<ModRef, String> {
        convert_reference(gm_variable_ref, &self.original_data.variables.variables, &self.modified_data.variables.variables)
    }

    pub fn convert_audio_ref_opt(&self, gm_audio_ref: &Option<GMRef<GMEmbeddedAudio>>) -> Result<Option<ModRef>, String> {
        convert_reference_optional(gm_audio_ref, &self.original_data.audios.audios, &self.modified_data.audios.audios)
    }
    pub fn convert_background_ref_opt(&self, gm_background_ref: &Option<GMRef<GMBackground>>) -> Result<Option<ModRef>, String> {
        convert_reference_optional(gm_background_ref, &self.original_data.backgrounds.backgrounds, &self.modified_data.backgrounds.backgrounds)
    }
    pub fn convert_code_ref_opt(&self, gm_code_ref: &Option<GMRef<GMCode>>) -> Result<Option<ModRef>, String> {
        convert_reference_optional(gm_code_ref, &self.original_data.codes.codes, &self.modified_data.codes.codes)
    }
    pub fn convert_game_object_ref_opt(&self, gm_game_object_ref: &Option<GMRef<GMGameObject>>) -> Result<Option<ModRef>, String> {
        convert_reference_optional(gm_game_object_ref, &self.original_data.game_objects.game_objects, &self.modified_data.game_objects.game_objects)
    }
    pub fn convert_sprite_ref_opt(&self, gm_sprite_ref: &Option<GMRef<GMSprite>>) -> Result<Option<ModRef>, String> {
        convert_reference_optional(gm_sprite_ref, &self.original_data.sprites.sprites, &self.modified_data.sprites.sprites)
    }
    pub fn convert_string_ref_opt(&self, gm_string_ref: &Option<GMRef<String>>) -> Result<Option<ModRef>, String> {
        convert_reference_optional(gm_string_ref, &self.original_data.strings.strings, &self.modified_data.strings.strings)
    }
    /// TODO make custom function for texture page items (since texture contents are not checked)
    pub fn convert_texture_ref_opt(&self, gm_texture_ref: &Option<GMRef<GMTexturePageItem>>) -> Result<Option<ModRef>, String> {
        convert_reference_optional(gm_texture_ref, &self.original_data.texture_page_items.texture_page_items, &self.modified_data.texture_page_items.texture_page_items)
    }
}


fn convert_reference<GM>(gm_reference: &GMRef<GM>, original_list: &[GM], modified_list: &[GM]) -> Result<ModRef, String> {
    // If reference index out of bounds in modified data; throw error.
    // This should never happen in healthy gm data; just being cautious that the mod will be fully functional.
    if gm_reference.index >= modified_list.len() as u32 {
        return Err(format!(
            "Could not resolve {} reference with GameMaker index {} in list with length {}; out of bounds",
            std::any::type_name_of_val(&gm_reference), gm_reference.index, modified_list.len(),
        ))
    }

    let original_length: u32 = original_list.len() as u32;
    if gm_reference.index >= original_length {
        // If reference index exists (isn't out of bounds) in modified data but not in original data,
        // then the element was newly added --> "Add" reference
        Ok(ModRef::Add(gm_reference.index - original_length))
    } else {
        // If reference index exists in original data (and modified data; assumes unordered lists never remove elements),
        // then the element is a reference to the gamemaker data the mod will later be loaded in.
        Ok(ModRef::Data(gm_reference.index))
    }
}


fn convert_reference_optional<GM>(gm_reference_optional: &Option<GMRef<GM>>, original_list: &[GM], modified_list: &[GM]) -> Result<Option<ModRef>, String> {
    match gm_reference_optional {
        Some(gm_reference) => Ok(Some(convert_reference(gm_reference, original_list, modified_list)?)),
        None => Ok(None),
    }
}

