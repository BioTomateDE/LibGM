// SPDX-License-Identifier: GPL-3.0-only

bitflags::bitflags! {
    /// Function classifications a data file can have.
    ///
    /// DOCME: are these just remenant from GM8? does the runner actually check these?
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct FunctionClassifications: u64 {
        const INTERNET = 0x1;
        const JOYSTICK = 0x2;
        const GAMEPAD = 0x4;
        const IMMERSION = 0x8;
        const SCREENGRAB = 0x10;
        const MATH = 0x20;
        const ACTION = 0x40;
        const MATRIX_D3D = 0x80;
        const D3D_MODEL = 0x100;
        const DATA_STRUCTURES = 0x200;
        const FILE = 0x400;
        const INI = 0x800;
        const FILENAME = 0x1000;
        const DIRECTORY = 0x2000;
        const ENVIRONMENT = 0x4000;
        // unused #1
        const HTTP = 0x10000;
        const ENCODING = 0x20000;
        const UI_DIALOG = 0x40000;
        const MOTION_PLANNING = 0x80000;
        const SHAPE_COLLISION = 0x10_0000;
        const INSTANCE = 0x20_0000;
        const ROOM = 0x40_0000;
        const GAME = 0x80_0000;
        const DISPLAY = 0x100_0000;
        const DEVICE = 0x200_0000;
        const WINDOW = 0x400_0000;
        const DRAW_COLOR = 0x800_0000;
        const TEXTURE = 0x1000_0000;
        const LAYER = 0x2000_0000;
        const STRING = 0x4000_0000;
        const TILES = 0x8000_0000;
        const SURFACE = 0x1_0000_0000;
        const SKELETON = 0x2_0000_0000;
        const IO = 0x4_0000_0000;
        const VARIABLES = 0x8_0000_0000;
        const ARRAY = 0x10_0000_0000;
        const EXTERNAL_CALL = 0x20_0000_0000;
        const NOTIFICATION = 0x40_0000_0000;
        const DATE = 0x80_0000_0000;
        const PARTICLE = 0x100_0000_0000;
        const SPRITE = 0x200_0000_0000;
        const CLICKABLE = 0x400_0000_0000;
        const LEGACY_SOUND = 0x800_0000_0000;
        const AUDIO = 0x1000_0000_0000;
        const EVENT = 0x2000_0000_0000;
        // unused #2
        const FREE_TYPE = 0x8000_0000_0000;
        const ANALYTICS = 0x1_0000_0000_0000;
        // unused #3
        // unused #4
        const ACHIEVEMENT = 0x8_0000_0000_0000;
        const CLOUD_SAVING = 0x10_0000_0000_0000;
        const ADS = 0x20_0000_0000_0000;
        const OS = 0x40_0000_0000_0000;
        const IN_APP_PURCHASES = 0x80_0000_0000_0000;
        const FACEBOOK = 0x100_0000_0000_0000;
        const PHYSICS = 0x200_0000_0000_0000;
        const FLASH_AA = 0x400_0000_0000_0000;
        const CONSOLE = 0x800_0000_0000_0000;
        const BUFFER = 0x1000_0000_0000_0000;
        const STEAM =   0x2000_0000_0000_0000;
        // unused #5
        const SHADERS = 0x4000_0000_0000_0000;
        const VERTEX_BUFFERS = 0x8000_0000_0000_0000;
    }
}
