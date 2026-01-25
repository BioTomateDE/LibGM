use crate::gamemaker::{
    elements::{
        animation_curve::GMAnimationCurve, background::GMBackground, font::GMFont,
        function::GMFunction, game_object::GMGameObject, particle_system::GMParticleSystem,
        path::GMPath, room::GMRoom, script::GMScript, sequence::GMSequence, shader::GMShader,
        sound::GMSound, sprite::GMSprite, timeline::GMTimeline,
    },
    reference::GMRef,
};

/// A modern (2023.something) reference to game assets.
/// Used with the `PushReference` (`pushref`) instruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetReference {
    Object(GMRef<GMGameObject>),
    Sprite(GMRef<GMSprite>),
    Sound(GMRef<GMSound>),
    Room(GMRef<GMRoom>),
    Path(GMRef<GMPath>),
    Script(GMRef<GMScript>),
    Font(GMRef<GMFont>),
    Timeline(GMRef<GMTimeline>),
    Shader(GMRef<GMShader>),
    Sequence(GMRef<GMSequence>),
    AnimCurve(GMRef<GMAnimationCurve>),
    ParticleSystem(GMRef<GMParticleSystem>),
    Background(GMRef<GMBackground>),
    RoomInstance(i32),
    /// Does not exist in UTMT.
    Function(GMRef<GMFunction>),
}

impl AssetReference {
    /// The normalized asset type of this asset reference, expressed as a number.
    ///
    /// This uses the 2024.4+ asset type.
    ///
    /// NOTE: Functions are not a "real" asset type??? they return 0xFF for now
    #[must_use]
    pub const fn asset_type(&self) -> u8 {
        match self {
            Self::Object(_) => 0,
            Self::Sprite(_) => 1,
            Self::Sound(_) => 2,
            Self::Room(_) => 3,
            Self::Path(_) => 4,
            Self::Script(_) => 5,
            Self::Font(_) => 6,
            Self::Timeline(_) => 7,
            Self::Shader(_) => 8,
            Self::Sequence(_) => 9,
            Self::AnimCurve(_) => 10,
            Self::ParticleSystem(_) => 11,
            Self::Background(_) => 13,
            Self::RoomInstance(_) => 14,
            Self::Function(_) => 0xFF,
        }
    }
}
