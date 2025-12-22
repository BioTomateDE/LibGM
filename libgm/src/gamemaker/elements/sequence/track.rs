pub mod keyframe;

pub use keyframe::{Keyframe, Keyframes};
use macros::num_enum;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMElement, animation_curve::GMAnimationCurve},
        serialize::builder::DataBuilder,
    },
    prelude::*,
    util::init::{num_enum_from, vec_with_capacity},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub model_name: String,
    pub name: String,
    pub builtin_name: BuiltinName,
    pub traits: Traits,
    pub is_creation_track: bool,
    pub tags: Vec<i32>,
    pub sub_tracks: Vec<Track>,
    pub keyframes: Keyframes,
    pub owned_resources: Vec<GMAnimationCurve>,
}

impl GMElement for Track {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let model_name: String = reader.read_gm_string()?;
        let name: String = reader.read_gm_string()?;
        let builtin_name: BuiltinName = num_enum_from(reader.read_i32()?)?;
        let traits: Traits = num_enum_from(reader.read_i32()?)?;
        let is_creation_track = reader.read_bool32()?;

        let tag_count = reader.read_count("Track Tag")?;
        let owned_resources_count = reader.read_count("Track Owned Resource")?;
        let track_count = reader.read_count("Track")?;

        let mut tags: Vec<i32> = vec_with_capacity(tag_count)?;
        for _ in 0..tag_count {
            tags.push(reader.read_i32()?);
        }

        let mut owned_resources: Vec<GMAnimationCurve> = vec_with_capacity(owned_resources_count)?;

        for _ in 0..owned_resources_count {
            let animcurve_str: String = reader.read_gm_string()?;
            if animcurve_str != "GMAnimCurve" {
                bail!(
                    "Expected owned resource thingy of Track to \
                    be \"GMAnimCurve\"; but found {:?} for Track {:?}",
                    animcurve_str,
                    name,
                );
            }
            owned_resources.push(GMAnimationCurve::deserialize(reader)?);
        }

        let mut sub_tracks: Vec<Self> = vec_with_capacity(track_count)?;
        for _ in 0..track_count {
            sub_tracks.push(Self::deserialize(reader)?);
        }

        let keyframes = match model_name.as_str() {
            "GMAudioTrack" => Keyframes::Audio(keyframe::Data::deserialize(reader)?),
            "GMInstanceTrack" => Keyframes::Instance(keyframe::Data::deserialize(reader)?),
            "GMGraphicTrack" => Keyframes::Graphic(keyframe::Data::deserialize(reader)?),
            "GMSequenceTrack" => Keyframes::Sequence(keyframe::Data::deserialize(reader)?),
            "GMSpriteFramesTrack" => Keyframes::SpriteFrames(keyframe::Data::deserialize(reader)?),
            "GMAssetTrack" => bail!("Asset Track not yet supported"),
            "GMBoolTrack" => Keyframes::Bool(keyframe::Data::deserialize(reader)?),
            "GMStringTrack" => Keyframes::String(keyframe::Data::deserialize(reader)?),
            "GMIntTrack" => bail!("Int Track not yet supported"),
            "GMColourTrack" | "GMRealTrack" => {
                Keyframes::Color(keyframe::color::KeyframesData::deserialize(reader)?)
            },
            "GMTextTrack" => Keyframes::Text(keyframe::Data::deserialize(reader)?),
            "GMParticleTrack" => Keyframes::Particle(keyframe::Data::deserialize(reader)?),
            other => bail!("Invalid Model Name {other:?} while parsing Track"),
        };

        Ok(Self {
            model_name,
            name,
            builtin_name,
            traits,
            is_creation_track,
            tags,
            sub_tracks,
            keyframes,
            owned_resources,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.model_name);
        builder.write_gm_string(&self.name);
        builder.write_i32(self.builtin_name.into());
        builder.write_i32(self.traits.into());
        builder.write_bool32(self.is_creation_track);
        builder.write_usize(self.tags.len())?;
        builder.write_usize(self.owned_resources.len())?;
        builder.write_usize(self.sub_tracks.len())?;
        for tag in &self.tags {
            builder.write_i32(*tag);
        }
        for animation_curve in &self.owned_resources {
            builder.write_gm_string("GMAnimCurve");
            animation_curve.serialize(builder)?;
        }
        for track in &self.sub_tracks {
            track.serialize(builder)?;
        }
        match &self.keyframes {
            Keyframes::Audio(k) => k.serialize(builder)?,
            Keyframes::Instance(k) => k.serialize(builder)?,
            Keyframes::Graphic(k) => k.serialize(builder)?,
            Keyframes::Sequence(k) => k.serialize(builder)?,
            Keyframes::SpriteFrames(k) => k.serialize(builder)?,
            Keyframes::Bool(k) => k.serialize(builder)?,
            Keyframes::String(k) => k.serialize(builder)?,
            Keyframes::Color(k) => k.serialize(builder)?,
            Keyframes::Text(k) => k.serialize(builder)?,
            Keyframes::Particle(k) => k.serialize(builder)?,
            Keyframes::BroadcastMessage(k) => k.serialize(builder)?,
        }
        Ok(())
    }
}

#[num_enum(i32)]
pub enum BuiltinName {
    /// No idea when/why this happens exactly
    None = 0,
    Gain = 5,
    Pitch = 6,
    Falloff = 7,
    RotationOrImageAngle = 8,
    BlendAdd = 9,
    BlendMultiplyOrImageBlend = 10,
    Mask = 12,
    Subject = 13,
    Position = 14,
    Scale = 15,
    Origin = 16,
    ImageSpeed = 17,
    ImageIndex = 18,
    FrameSize = 20,
    CharacterSpacing = 21,
    LineSpacing = 22,
    ParagraphSpacing = 23,
}

#[num_enum(i32)]
pub enum Traits {
    None,
    ChildrenIgnoreOrigin,
}
