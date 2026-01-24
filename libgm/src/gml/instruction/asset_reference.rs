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
#[derive(Debug, Clone, PartialEq)]
pub enum AssetReference {
    Object(GMRef<GMGameObject>),
    Sprite(GMRef<GMSprite>),
    Sound(GMRef<GMSound>),
    Room(GMRef<GMRoom>),
    Background(GMRef<GMBackground>),
    Path(GMRef<GMPath>),
    Script(GMRef<GMScript>),
    Font(GMRef<GMFont>),
    Timeline(GMRef<GMTimeline>),
    Shader(GMRef<GMShader>),
    Sequence(GMRef<GMSequence>),
    AnimCurve(GMRef<GMAnimationCurve>),
    ParticleSystem(GMRef<GMParticleSystem>),
    RoomInstance(i32),
    /// Does not exist in UTMT.
    Function(GMRef<GMFunction>),
}
