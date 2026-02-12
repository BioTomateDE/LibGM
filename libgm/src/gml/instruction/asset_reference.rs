use crate::{
    gamemaker::elements::{
        animation_curve::GMAnimationCurve, background::GMBackground, font::GMFont,
        function::GMFunction, game_object::GMGameObject, particle_system::GMParticleSystem,
        path::GMPath, room::GMRoom, script::GMScript, sequence::GMSequence, shader::GMShader,
        sound::GMSound, sprite::GMSprite, timeline::GMTimeline,
    },
    prelude::*,
};

/// A modern (2023.something) reference to game assets.
/// Used with the `PushReference` (`pushref`) instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    /// Weird special reference since functions use reference chains.
    /// Functions therefore do not have an actual asset type byte.
    Function(GMRef<GMFunction>),
}

impl AssetReference {
    /// Constructs a new asset reference based on the normalized serialized form.
    ///
    /// This uses the 2024.4+ format.
    ///
    /// Note that function references have to be constructed separately.
    pub fn parse(raw: u32) -> Result<Self> {
        let ty = (raw >> 24) as u8;
        let id = raw & 0xFF_FFFF;
        Ok(match ty {
            0 => Self::Object(GMRef::new(id)),
            1 => Self::Sprite(GMRef::new(id)),
            2 => Self::Sound(GMRef::new(id)),
            3 => Self::Room(GMRef::new(id)),
            4 => Self::Path(GMRef::new(id)),
            5 => Self::Script(GMRef::new(id)),
            6 => Self::Font(GMRef::new(id)),
            7 => Self::Timeline(GMRef::new(id)),
            8 => Self::Shader(GMRef::new(id)),
            9 => Self::Sequence(GMRef::new(id)),
            10 => Self::AnimCurve(GMRef::new(id)),
            11 => Self::ParticleSystem(GMRef::new(id)),
            13 => Self::Background(GMRef::new(id)),
            14 => Self::RoomInstance(id as i32),
            _ => bail!("Invalid asset type {ty}"),
        })
    }

    /// Constructs a new asset reference based on the old (pre 2024.4) serialized form.
    ///
    /// Note that function references have to be constructed separately.
    pub fn parse_old(raw: u32) -> Result<Self> {
        let ty = (raw >> 24) as u8;
        let id = raw & 0xFF_FFFF;
        Ok(match ty {
            0 => Self::Object(GMRef::new(id)),
            1 => Self::Sprite(GMRef::new(id)),
            2 => Self::Sound(GMRef::new(id)),
            3 => Self::Room(GMRef::new(id)),
            4 => Self::Background(GMRef::new(id)),
            5 => Self::Path(GMRef::new(id)),
            6 => Self::Script(GMRef::new(id)),
            7 => Self::Font(GMRef::new(id)),
            8 => Self::Timeline(GMRef::new(id)),
            10 => Self::Shader(GMRef::new(id)),
            11 => Self::Sequence(GMRef::new(id)),
            12 => Self::AnimCurve(GMRef::new(id)),
            13 => Self::ParticleSystem(GMRef::new(id)),
            14 => Self::RoomInstance(id as i32),
            _ => bail!("Invalid asset type {ty}"),
        })
    }

    /// The normalized asset type of this asset reference, expressed as a number.
    ///
    /// This uses the 2024.4+ asset type.
    ///
    /// NOTE: Functions return the `Script` asset type since they don't "actually have one".
    #[must_use]
    pub const fn asset_type(self) -> u8 {
        match self {
            Self::Object(_) => 0,
            Self::Sprite(_) => 1,
            Self::Sound(_) => 2,
            Self::Room(_) => 3,
            Self::Path(_) => 4,
            Self::Script(_) | Self::Function(_) => 5,
            Self::Font(_) => 6,
            Self::Timeline(_) => 7,
            Self::Shader(_) => 8,
            Self::Sequence(_) => 9,
            Self::AnimCurve(_) => 10,
            Self::ParticleSystem(_) => 11,
            Self::Background(_) => 13,
            Self::RoomInstance(_) => 14,
        }
    }

    /// The old (pre 2024.4) asset type of this asset reference, expressed as a number.
    ///
    /// NOTE: Functions return the `Script` asset type since they don't "actually have one".
    #[must_use]
    pub const fn asset_type_old(self) -> u8 {
        match self {
            Self::Object(_) => 0,
            Self::Sprite(_) => 1,
            Self::Sound(_) => 2,
            Self::Room(_) => 3,
            Self::Background(_) => 4,
            Self::Path(_) => 5,
            Self::Script(_) | Self::Function(_) => 6,
            Self::Font(_) => 7,
            Self::Timeline(_) => 8,
            Self::Shader(_) => 10,
            Self::Sequence(_) => 11,
            Self::AnimCurve(_) => 12,
            Self::ParticleSystem(_) => 13,
            Self::RoomInstance(_) => 14,
        }
    }

    /// The u24 asset id (aka index) of this asset reference.
    #[must_use]
    pub const fn asset_id(self) -> u32 {
        match self {
            Self::Object(gm_ref) => gm_ref.index,
            Self::Sprite(gm_ref) => gm_ref.index,
            Self::Sound(gm_ref) => gm_ref.index,
            Self::Room(gm_ref) => gm_ref.index,
            Self::Background(gm_ref) => gm_ref.index,
            Self::Path(gm_ref) => gm_ref.index,
            Self::Script(gm_ref) => gm_ref.index,
            Self::Font(gm_ref) => gm_ref.index,
            Self::Timeline(gm_ref) => gm_ref.index,
            Self::Shader(gm_ref) => gm_ref.index,
            Self::Sequence(gm_ref) => gm_ref.index,
            Self::AnimCurve(gm_ref) => gm_ref.index,
            Self::ParticleSystem(gm_ref) => gm_ref.index,
            Self::RoomInstance(integer) => integer as u32,
            Self::Function(gm_ref) => gm_ref.index,
        }
    }

    /// The normalized (2024.4+) serialized form of this asset reference.
    #[must_use]
    pub const fn build(self) -> u32 {
        let id = self.asset_id() & 0xFF_FFFF;
        let ty = self.asset_type() as u32;
        id & (ty << 24)
    }

    /// The old (pre 2024.4) serialized form of this asset reference.
    #[must_use]
    pub const fn build_old(self) -> u32 {
        let id = self.asset_id() & 0xFF_FFFF;
        let ty = self.asset_type_old() as u32;
        id & (ty << 24)
    }
}
