mod printing;
mod deserialize;
mod serialize;
mod export_mod;
mod debug_utils;

pub use deserialize::all::{read_data_file, parse_data_file, GMData};

pub mod gm {
    pub use crate::deserialize::backgrounds::{
        GMBackgrounds, 
        GMBackground,
        GMBackgroundGMS2Data,
    };
    pub use crate::deserialize::code::{
        GMOpcode, 
        GMCodeBytecode15, 
        GMInstruction, 
        GMBreakInstruction, 
        GMCallInstruction, 
        GMCode, 
        GMCodes, 
        GMCodeVariable, 
        GMComparisonInstruction, 
        GMComparisonType, 
        GMDataType, 
        GMDoubleTypeInstruction, 
        GMGotoInstruction, 
        GMInstanceType, 
        GMPopInstruction, 
        GMPushInstruction, 
        GMSingleTypeInstruction,
        GMValue,
        GMVariableType,
    };
    pub use crate::deserialize::embedded_audio::{
        GMEmbeddedAudios,
        GMEmbeddedAudio,
    };
    pub use crate::deserialize::embedded_textures::{
        GMEmbeddedTexture,
    };
    pub use crate::deserialize::fonts::{
        GMFonts,
        GMFont,
        GMFontGlyph,
        GMFontGlyphKerning,
    };
    pub use crate::deserialize::functions::{
        GMFunctions,
        GMFunction,
        GMCodeLocal,
        GMCodeLocalVariable,
    };
    pub use crate::deserialize::game_objects::{
        GMGameObjects,
        GMGameObject,
        GMGameObjectCollisionShape,
        GMGameObjectEvent,
        GMGameObjectEventAction,
    };
    pub use crate::deserialize::general_info::{
        GMGeneralInfo,
        GMGeneralInfoFlags,
        GMVersion,
        GMFunctionClassifications,
        GMOptions,
        GMOptionsConstant,
        GMOptionsFlags,
        GMOptionsWindowColor,
    };
    pub use crate::deserialize::paths::{
        GMPaths,
        GMPath,
        GMPathPoint,
    };
    pub use crate::deserialize::rooms::{
        GMRooms,
        GMRoom,
        GMRoomLayer,
        GMRoomLayerType,
        GMRoomBackground,
        GMRoomFlags,
        GMRoomGameObject,
        GMRoomTile,
        GMRoomTileTexture,
        GMRoomView,
    };
    // TODO continue
}


