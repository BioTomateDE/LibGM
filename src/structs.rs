use std::collections::HashMap;
use chrono::{DateTime, Utc};

pub struct UTOptions {

}

#[derive(Clone)]
pub struct UTGeneralInfo {
    pub is_debugger_disabled: bool,
    pub bytecode_version: u8,
    pub unknown_value: u16,
    pub game_file_name: String,
    pub config: String,
    pub last_object_id: u32,
    pub last_tile_id: u32,
    pub game_id: u32,
    pub directplay_guid: uuid::Uuid,
    pub game_name: String,
    pub major_version: u32,
    pub minor_version: u32,
    pub release_version: u32,
    pub stable_version: u32,
    pub default_window_width: u32,
    pub default_window_height: u32,
    pub flags: UTGeneralInfoFlags,
    pub license: [u8; 16],
    pub timestamp_created: DateTime<Utc>,
    pub display_name: String,
    pub active_targets: u64,
    pub function_classifications: UTFunctionClassifications,
    pub steam_appid: u32,
    pub debugger_port: u16,
    pub room_order: Vec<u32>,
}
#[derive(Clone)]
pub struct UTGeneralInfoFlags {
    // taken from https://github.com/UnderminersTeam/UndertaleModTool/blob/master/UndertaleModLib/Models/UndertaleGeneralInfo.cs
    pub fullscreen: bool,
    pub sync_vertex1: bool,
    pub sync_vertex2: bool,
    pub sync_vertex3: bool,
    pub interpolate: bool,
    pub scale: bool,
    pub show_cursor: bool,
    pub sizeable: bool,
    pub screen_key: bool,
    pub studio_version_b1: bool,
    pub studio_version_b2: bool,
    pub studio_version_b3: bool,
    // studio_version_mask
    pub steam_enabled: bool,
    pub local_data_enabled: bool,
    pub borderless_window: bool,
    pub javascript_mode: bool,
    // license_exclusions: bool,
}

#[derive(Clone)]
pub struct UTFunctionClassifications {
    pub none: bool,
    pub internet: bool,
    pub joystick: bool,
    pub gamepad: bool,
    pub immersion: bool,
    pub screengrab: bool,
    pub math: bool,
    pub action: bool,
    pub matrix_d3d: bool,
    pub d3dmodel: bool,
    pub data_structures: bool,
    pub file: bool,
    pub ini: bool,
    pub filename: bool,
    pub directory: bool,
    pub environment: bool,
    pub _unused1: bool,
    pub http: bool,
    pub encoding: bool,
    pub uidialog: bool,
    pub motion_planning: bool,
    pub shape_collision: bool,
    pub instance: bool,
    pub room: bool,
    pub game: bool,
    pub display: bool,
    pub device: bool,
    pub window: bool,
    pub draw_color: bool,
    pub texture: bool,
    pub layer: bool,
    pub string: bool,
    pub tiles: bool,
    pub surface: bool,
    pub skeleton: bool,
    pub io: bool,
    pub variables: bool,
    pub array: bool,
    pub external_call: bool,
    pub notification: bool,
    pub date: bool,
    pub particle: bool,
    pub sprite: bool,
    pub clickable: bool,
    pub legacy_sound: bool,
    pub audio: bool,
    pub event: bool,
    pub _unused2: bool,
    pub free_type: bool,
    pub analytics: bool,
    pub unused3: bool,
    pub unused4: bool,
    pub achievement: bool,
    pub cloud_saving: bool,
    pub ads: bool,
    pub os: bool,
    pub iap: bool,
    pub facebook: bool,
    pub physics: bool,
    pub flash_aa: bool,
    pub console: bool,
    pub buffer: bool,
    pub steam: bool,
    pub _unused3: bool,
    pub shaders: bool,
    pub vertex_buffers: bool,
}

pub struct UTData {
    // pub chunks: HashMap<String, UTChunk>, // remove when all chunks parsed properly
    pub strings: HashMap<u32, String>,    // STRG
    pub general_info: UTGeneralInfo,      // GEN8
}

pub struct DataChange {
    pub index: usize,
    pub content: Vec<u8>,
    pub delete: bool,
}

impl DataChange {
    pub fn apply(&self, data: Vec<u8>) {
        if self.delete {
            let _ = self.__delete(data);
        } else {
            self.__insert(data)
        }
    }

    fn __insert(&self, mut data: Vec<u8>) {
        data.splice(self.index..self.index, self.content.clone());
    }

    fn __delete(&self, mut data: Vec<u8>) -> Result<(), String> {
        let len: usize = self.content.len();
        if data[self.index..self.index + len] != self.content {
            return Err(format!(
                "Could not delete {} bytes at position {} because they dont exist in the code at the specified location!",
                len, self.index
            ));
        }
        data.splice(self.index..self.index + len, []);
        Ok(())
    }
}
