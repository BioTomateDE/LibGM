use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::gamemaker::elements::general_info::{GMFunctionClassifications, GMGeneralInfo, GMGeneralInfoFlags};
use crate::modding::export::{edit_field, edit_field_convert, flag_field, ModExporter, ModRef};
use crate::modding::ordered_list::{export_changes_ordered_list, DataChange};

macro_rules! prevent_enabling {
    ($original:expr, $modified:expr, $field:ident) => {{
        if !$original.$field && $modified.$field {
            return Err(format!(
                "Enabling function classification \"{}\" is not allowed for security reasons!\n\
                You are only allowed to use functions that are already unlocked in the original game. \
                If you have a good reason to enable this function classification for modding, open a GitHub issue regarding this.",
                stringify!($field),
            ))
        }
    }};
}

// macro_rules! prevent_changing {
//     ($original:expr, $modified:expr, $field:ident, $name:expr) => {{
//         if $original.$field != $modified.$field {
//             return Err(format!("Changing general info field {} is not allowed!", $name))
//         }
//     }};
// }


#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditGeneralInfo {
    pub debugger_enabled: Option<bool>,
    pub game_name: Option<ModRef>,      // String ref
    pub file_name: Option<ModRef>,     // String ref
    pub gamemaker_config_string: Option<ModRef>,     // String ref
    pub game_id: Option<u32>,
    pub creation_timestamp: Option<DateTime<Utc>>,
    pub default_window_width: Option<u32>,
    pub default_window_height: Option<u32>,
    pub default_window_title: Option<ModRef>,   // String ref
    pub directplay_guid: Option<uuid::Uuid>,
    pub steam_app_id: Option<i32>,
    pub debugger_port: Option<u32>,
    pub flags: EditGeneralInfoFlags,
    pub function_classifications: EditFunctionClassifications,
    pub room_order: Vec<DataChange<ModRef>>,    // GMRoom reference
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditGeneralInfoFlags {
    /// Whether the game starts in fullscreen mode
    /// Corresponds to "Start in Fullscreen Mode" in Global Game Settings
    pub fullscreen: Option<bool>,

    /// Vertex synchronization option 1 (legacy OpenGL setting)
    /// Related to "Use synchronization to avoid tearing" in older versions
    pub sync_vertex1: Option<bool>,

    /// Vertex synchronization option 2 (legacy OpenGL setting)
    pub sync_vertex2: Option<bool>,

    /// Vertex synchronization option 3 (legacy OpenGL setting)
    pub sync_vertex3: Option<bool>,

    /// Whether to interpolate colors between pixels
    /// Corresponds to "Interpolate colors between pixels" in Graphics settings
    pub interpolate: Option<bool>,

    /// Whether to allow scaling the game window
    /// "Allow window scaling" in Graphics settings
    pub scale: Option<bool>,

    /// Whether to display the cursor in game
    /// "Display cursor" in Graphics settings
    pub show_cursor: Option<bool>,

    /// Whether the window is resizable
    /// "Allow window resize" in Graphics settings
    pub sizeable: Option<bool>,

    /// Whether the screen/keyboard is locked to game
    /// Obscure setting related to focus locking
    pub screen_key: Option<bool>,

    /// Studio version bitflag 1 (internal use)
    pub studio_version_b1: Option<bool>,

    /// Studio version bitflag 2 (internal use)
    pub studio_version_b2: Option<bool>,

    /// Studio version bitflag 3 (internal use)
    pub studio_version_b3: Option<bool>,

    /// Whether Steam integration is enabled
    /// "Enable Steam" in Platform Settings
    pub steam_enabled: Option<bool>,

    /// Whether local data storage is allowed
    /// "Allow local data storage" in Platform Settings
    pub local_data_enabled: Option<bool>,

    /// Whether to use borderless window mode
    /// "Borderless window" in Graphics settings
    pub borderless_window: Option<bool>,

    /// Whether targeting HTML5/JavaScript export
    /// Internal flag for JavaScript target
    pub javascript_mode: Option<bool>,

    /// Whether license restrictions apply
    /// Internal license management flag
    pub license_exclusions: Option<bool>,
}


/// Field docstrings are guessed; not taken from UTMT or GameMaker docs.
/// ___ 
/// Keep in mind that the disabled fields only prevent these flags/classifications from being *toggled* (most importantly, enabled).
/// If the target game already has them enabled, then the mod's code can execute these functions just fine.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditFunctionClassifications {
    /// Basic functions with no special classification
    pub none: Option<bool>,

    /// Joystick input (old-school analog joysticks)
    pub joystick: Option<bool>,

    /// Modern gamepad/controller input (XInput/DirectInput)
    /// Differs from joystick in supporting vibration, triggers, etc.
    pub gamepad: Option<bool>,

    /// Haptic feedback/force feedback systems
    pub immersion: Option<bool>,

    /// Screen capture functionality
    ///> SECURITY: Could be used to spy on gameplay
    pub screen_capture: Option<bool>,

    /// Mathematical operations
    pub math: Option<bool>,

    /// Game action management functions
    pub action: Option<bool>,

    /// Direct3D matrix operations
    pub matrix_d3d: Option<bool>,

    /// 3D model rendering
    pub direct3d_model_rendering: Option<bool>,

    /// Data structure operations (lists, maps, grids)
    pub data_structures: Option<bool>,

    /// File I/O operations
    ///> SECURITY: High risk - disable or sandbox for mods
    pub file: Option<bool>,

    /// INI file operations
    ///> SECURITY: Moderate risk - can access filesystem
    pub ini: Option<bool>,

    /// Filename/path manipulation
    ///> SECURITY: Moderate risk - path traversal possible
    pub filename: Option<bool>,

    /// Directory operations
    ///> SECURITY: High risk - filesystem access
    pub directory: Option<bool>,

    /// System environment variables
    ///> SECURITY: High risk - can leak system info
    pub environment: Option<bool>,

    /// HTTP networking
    ///> SECURITY: CRITICAL - disable for untrusted mods
    pub http: Option<bool>,

    /// Text encoding/decoding
    pub encoding: Option<bool>,

    /// System dialog boxes
    pub ui_dialog: Option<bool>,

    /// Pathfinding/movement algorithms
    pub motion_planning: Option<bool>,

    /// Collision detection
    pub shape_collision: Option<bool>,

    /// Instance manipulation functions (create/destroy instances)
    pub instance: Option<bool>,

    /// Room management functions
    pub room: Option<bool>,

    /// Core game control functions
    pub game: Option<bool>,

    /// Display/graphics control
    pub display: Option<bool>,

    /// Input device handling
    pub device: Option<bool>,

    /// Window management functions
    ///> SECURITY: Could fake UI elements in borderless mode
    pub window: Option<bool>,

    /// Color/drawing operations
    pub draw_color: Option<bool>,

    /// Texture manipulation
    pub texture: Option<bool>,

    /// Layer management
    pub layer: Option<bool>,

    /// String manipulation
    pub string: Option<bool>,

    /// Tilemap operations
    pub tiles: Option<bool>,

    /// Surface rendering functions
    ///> SECURITY: Could be used for overlay attacks
    pub surface: Option<bool>,

    /// Skeletal animation functions
    pub skeleton: Option<bool>,

    /// General I/O operations
    ///> SECURITY: High risk - disable for mods
    /// can access files (could steal data or ransomware)
    pub io: Option<bool>,

    /// Variable manipulation
    pub variables: Option<bool>,

    /// Array operations
    pub array: Option<bool>,

    /// External function calls (DLL/FFI)
    ///> SECURITY: CRITICAL - disable completely for mods
    /// yeah this is never getting enabled
    pub external_call: Option<bool>,

    /// System notifications/alerts
    ///> SECURITY: Could fake system messages
    pub notifications: Option<bool>,

    /// Date/time functions
    pub time_and_date: Option<bool>,

    /// Particle system control
    pub particle: Option<bool>,

    /// Sprite manipulation
    pub sprite: Option<bool>,

    /// Clickable object handling
    pub clickable_object_handling: Option<bool>,

    /// Legacy sound system
    pub legacy_sound: Option<bool>,

    /// Modern audio system
    pub audio: Option<bool>,

    /// Event management
    pub event: Option<bool>,

    /// Font rendering (FreeType)
    pub free_type: Option<bool>,

    /// Analytics reporting
    ///> SECURITY: Privacy concerns - anonymize if enabled
    pub analytics: Option<bool>,

    /// Achievement systems
    pub achievements: Option<bool>,

    /// Cloud save functionality
    ///> SECURITY: Could corrupt save data
    pub cloud_saving: Option<bool>,
    
    /// Advertising systems
    ///> SECURITY: Could inject malicious ads
    pub ads: Option<bool>,

    /// Operating system functions
    /// https://manual.gamemaker.io/lts/en/GameMaker_Language/GML_Reference/OS_And_Compiler/OS_And_Compiler.htm (2025-07-13)
    /// general os info, environment variables, clipboard
    pub os: Option<bool>,

    /// In-app purchases
    ///> SECURITY: Financial risk - disable completely
    /// deprecated according to https://manual.gamemaker.io/lts/en/GameMaker_Language/GML_Reference/In_App_Purchases/In_App_Purchases.htm (2025-07-13)
    pub in_app_purchases: Option<bool>,

    /// Facebook integration
    ///> SECURITY: Privacy risk - disable
    /// honestly not a big concern but mods don't need to enable ts either
    pub facebook: Option<bool>,

    /// Physics system
    pub physics: Option<bool>,

    /// Flash/Anti-aliasing controls
    pub flash_anti_alias: Option<bool>,

    /// Console/debug output
    ///> SECURITY: Could expose sensitive data
    pub console: Option<bool>,

    /// Buffer operations
    ///> SECURITY: Memory manipulation risk
    pub buffer: Option<bool>,

    /// Steamworks integration
    ///> SECURITY: API key exposure risk
    pub steam: Option<bool>,

    /// Shader programs
    ///> SECURITY: Could crash GPU drivers
    pub shaders: Option<bool>,

    /// Vertex buffer operations
    pub vertex_buffers: Option<bool>,
}

impl ModExporter<'_, '_> {
    pub fn export_general_info(&self) -> Result<EditGeneralInfo, String> {
        let o: &GMGeneralInfo = &self.original_data.general_info;
        let m: &GMGeneralInfo = &self.modified_data.general_info;

        // prevent_changing!(o, m, bytecode_version, "Bytecode Version");
        // prevent_changing!(o, m, unknown_value, "Unknown Value");
        // prevent_changing!(o, m, directplay_guid, "Directplay GUID");
        // prevent_changing!(o, m, version, "GameMaker version");
        // prevent_changing!(o, m, license_crc32, "Licence (CRC32)");
        // prevent_changing!(o, m, license_md5, "Licence (MD5)");
        // prevent_changing!(o, m, active_targets, "Active Targets");
        // TODO: find function usages in code when loading; i dont trust the runner
        
        Ok(EditGeneralInfo {
            debugger_enabled: edit_field(&o.is_debugger_disabled, &m.is_debugger_disabled),
            game_name: edit_field_convert(&o.game_name, &m.game_name, |r| self.convert_string_ref(r))?,
            file_name: edit_field_convert(&o.game_file_name, &m.game_file_name, |r| self.convert_string_ref(r))?,
            gamemaker_config_string: edit_field_convert(&o.config, &m.config, |r| self.convert_string_ref(r))?,
            game_id: edit_field(&o.game_id, &m.game_id),
            creation_timestamp: edit_field(&o.timestamp_created, &m.timestamp_created),
            default_window_width: edit_field(&o.default_window_width, &m.default_window_width),
            default_window_height: edit_field(&o.default_window_height, &m.default_window_height),
            default_window_title: edit_field_convert(&o.display_name, &m.display_name, |r| self.convert_string_ref(r))?,
            directplay_guid: edit_field(&o.directplay_guid, &m.directplay_guid),
            steam_app_id: edit_field(&o.steam_appid, &m.steam_appid),
            debugger_port: edit_field(&o.debugger_port, &m.debugger_port).flatten(),
            flags: edit_flags(&o.flags, &m.flags),
            function_classifications: edit_function_classifications(&o.function_classifications, &m.function_classifications)?,
            room_order: export_changes_ordered_list(&o.room_order, &m.room_order, |i| self.convert_room_ref(i))?,
        })
    }
}

fn edit_flags(o: &GMGeneralInfoFlags, m: &GMGeneralInfoFlags) -> EditGeneralInfoFlags {
    EditGeneralInfoFlags {
        fullscreen: flag_field(o.fullscreen, m.fullscreen),
        sync_vertex1: flag_field(o.sync_vertex1, m.sync_vertex1),
        sync_vertex2: flag_field(o.sync_vertex2, m.sync_vertex2),
        sync_vertex3: flag_field(o.sync_vertex3, m.sync_vertex3),
        interpolate: flag_field(o.interpolate, m.interpolate),
        scale: flag_field(o.scale, m.scale),
        show_cursor: flag_field(o.show_cursor, m.show_cursor),
        sizeable: flag_field(o.sizeable, m.sizeable),
        screen_key: flag_field(o.screen_key, m.screen_key),
        studio_version_b1: flag_field(o.studio_version_b1, m.studio_version_b1),
        studio_version_b2: flag_field(o.studio_version_b2, m.studio_version_b2),
        studio_version_b3: flag_field(o.studio_version_b3, m.studio_version_b3),
        steam_enabled: flag_field(o.steam_enabled, m.steam_enabled),
        local_data_enabled: flag_field(o.local_data_enabled, m.local_data_enabled),
        borderless_window: flag_field(o.borderless_window, m.borderless_window),
        javascript_mode: flag_field(o.javascript_mode, m.javascript_mode),
        license_exclusions: flag_field(o.license_exclusions, m.license_exclusions),
    }
}

fn edit_function_classifications(o: &GMFunctionClassifications, m: &GMFunctionClassifications) -> Result<EditFunctionClassifications, String> {
    prevent_enabling!(o, m, http);
    // prevent_enabling!(o, m, io);
    prevent_enabling!(o, m, external_call);
    prevent_enabling!(o, m, analytics);
    prevent_enabling!(o, m, ads);
    // prevent_enabling!(o, m, os);
    prevent_enabling!(o, m, iap);
    prevent_enabling!(o, m, facebook);

    Ok(EditFunctionClassifications {
        none: flag_field(o.none, m.none),
        joystick: flag_field(o.joystick, m.joystick),
        gamepad: flag_field(o.gamepad, m.gamepad),
        immersion: flag_field(o.immersion, m.immersion),
        screen_capture: flag_field(o.screengrab, m.screengrab),
        math: flag_field(o.math, m.math),
        action: flag_field(o.action, m.action),
        matrix_d3d: flag_field(o.matrix_d3d, m.matrix_d3d),
        direct3d_model_rendering: flag_field(o.d3dmodel, m.d3dmodel),
        data_structures: flag_field(o.data_structures, m.data_structures),
        file: flag_field(o.file, m.file),
        ini: flag_field(o.ini, m.ini),
        filename: flag_field(o.filename, m.filename),
        directory: flag_field(o.directory, m.directory),
        environment: flag_field(o.environment, m.environment),
        http: flag_field(o.environment, m.environment),
        encoding: flag_field(o.encoding, m.encoding),
        ui_dialog: flag_field(o.uidialog, m.uidialog),
        motion_planning: flag_field(o.motion_planning, m.motion_planning),
        shape_collision: flag_field(o.shape_collision, m.shape_collision),
        instance: flag_field(o.instance, m.instance),
        room: flag_field(o.room, m.room),
        game: flag_field(o.game, m.game),
        display: flag_field(o.display, m.display),
        device: flag_field(o.device, m.device),
        window: flag_field(o.window, m.window),
        draw_color: flag_field(o.draw_color, m.draw_color),
        texture: flag_field(o.texture, m.texture),
        layer: flag_field(o.layer, m.layer),
        string: flag_field(o.string, m.string),
        tiles: flag_field(o.tiles, m.tiles),
        surface: flag_field(o.surface, m.surface),
        skeleton: flag_field(o.skeleton, m.skeleton),
        io: flag_field(o.io, m.io),
        variables: flag_field(o.variables, m.variables),
        array: flag_field(o.array, m.array),
        external_call: None,    // never
        notifications: flag_field(o.notification, m.notification),
        time_and_date: flag_field(o.date, m.date),
        particle: flag_field(o.particle, m.particle),
        sprite: flag_field(o.sprite, m.sprite),
        clickable_object_handling: flag_field(o.clickable, m.clickable),
        legacy_sound: flag_field(o.legacy_sound, m.legacy_sound),
        audio: flag_field(o.audio, m.audio),
        event: flag_field(o.event, m.event),
        free_type: flag_field(o.free_type, m.free_type),
        analytics: flag_field(o.analytics, m.analytics),
        achievements: flag_field(o.achievement, m.achievement),
        cloud_saving: flag_field(o.cloud_saving, m.cloud_saving),
        ads: flag_field(o.ads, m.ads),
        os: flag_field(o.os, m.os),
        in_app_purchases: flag_field(o.iap, m.iap),
        facebook: flag_field(o.facebook, m.facebook),
        physics: flag_field(o.physics, m.physics),
        flash_anti_alias: flag_field(o.flash_aa, m.flash_aa),
        console: flag_field(o.console, m.console),
        buffer: flag_field(o.buffer, m.buffer),
        steam: flag_field(o.steam, m.steam),
        shaders: flag_field(o.shaders, m.shaders),
        vertex_buffers: flag_field(o.vertex_buffers, m.vertex_buffers),
    })
}

