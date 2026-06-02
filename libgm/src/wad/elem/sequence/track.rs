// SPDX-License-Identifier: GPL-3.0-only
pub mod keyframe;

pub use keyframe::Keyframe;
pub use keyframe::Keyframes;

use crate::gm_enum::gm_enum;
use crate::prelude::*;
use crate::util::bitfield::bitfield_struct;
use crate::util::init::vec_with_capacity;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::GMElement;
use crate::wad::elem::animation_curve::GMAnimationCurve;
use crate::wad::parse::reader::DataReader;

#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    /// Name for the type/model of track, such as `GMGroupTrack`,
    /// `GMInstanceTrack`, `GMRealTrack`, etc.
    pub model_name: GMRef<String>,

    /// Name of the track. Can be user-assigned or the name of a property or
    /// asset.
    pub name: GMRef<String>,

    /// Builtin name for the track, representing the type of property, or 0 if
    /// not applicable.
    pub builtin_name: BuiltinName,

    /// Traits for the track.
    pub flags: Flags,

    /// Whether the track is a creation track (whatever that means).
    pub is_creation_track: bool,

    /// Tags for the track (which might not be used?).
    pub tags: Vec<i32>,

    /// List of sub-tracks of this track.
    pub sub_tracks: Vec<Self>,

    /// Keyframe store of this track.
    pub keyframes: Keyframes,

    /// Owned resources of this track (such as animation curves).
    pub owned_resources: Vec<GMAnimationCurve>,

    // "GMAnimCurve"
    animcurve_string: GMRef<String>,
}

impl GMElement for Track {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let model_name_str: &str = force_read_string(reader)?;
        reader.cur_pos -= 4;
        let model_name: GMRef<String> = reader.read_gm_string()?;

        let name: GMRef<String> = reader.read_gm_string()?;
        let builtin_name: BuiltinName = reader.read_enum()?;
        let flags = Flags::deserialize(reader)?;
        let is_creation_track = reader.read_bool32()?;

        let tag_count = reader.read_count("Track Tag")?;
        let owned_resources_count = reader.read_count("Track Owned Resource")?;
        let track_count = reader.read_count("Track")?;

        let mut tags: Vec<i32> = vec_with_capacity(tag_count)?;
        for _ in 0..tag_count {
            tags.push(reader.read_i32()?);
        }

        let mut owned_resources: Vec<GMAnimationCurve> = vec_with_capacity(owned_resources_count)?;
        let mut animcurve_string = GMRef::none();

        for _ in 0..owned_resources_count {
            let string: &str = force_read_string(reader)?;
            animcurve_string = reader.read_gm_string()?;
            if string != "GMAnimCurve" {
                bail!(
                    "Expected owned resource thingy of Track to be \"GMAnimCurve\"; but found \
                     {:?} for Track {:?}",
                    animcurve_string,
                    name,
                );
            }
            owned_resources.push(GMAnimationCurve::deserialize(reader)?);
        }

        let mut sub_tracks: Vec<Self> = vec_with_capacity(track_count)?;
        for _ in 0..track_count {
            sub_tracks.push(Self::deserialize(reader)?);
        }

        let keyframes = match model_name_str {
            "GMAudioTrack" => Keyframes::Audio(keyframe::Data::deserialize(reader)?),
            "GMInstanceTrack" => Keyframes::Instance(keyframe::Data::deserialize(reader)?),
            "GMGraphicTrack" => Keyframes::Graphic(keyframe::Data::deserialize(reader)?),
            "GMSequenceTrack" => Keyframes::Sequence(keyframe::Data::deserialize(reader)?),
            "GMSpriteFramesTrack" => Keyframes::SpriteFrames(keyframe::Data::deserialize(reader)?),
            "GMAssetTrack" => bail!("Asset Track not yet supported"),
            "GMBoolTrack" => Keyframes::Bool(keyframe::Data::deserialize(reader)?),
            "GMStringTrack" => Keyframes::String(keyframe::Data::deserialize(reader)?),
            "GMIntTrack" => bail!("Int Track not yet supported"),
            "GMColourTrack" => {
                Keyframes::Color(keyframe::color::KeyframesData::deserialize(reader)?)
            }
            "GMRealTrack" => Keyframes::Real(keyframe::color::KeyframesData::deserialize(reader)?),
            "GMTextTrack" => Keyframes::Text(keyframe::Data::deserialize(reader)?),
            "GMParticleTrack" => Keyframes::Particle(keyframe::Data::deserialize(reader)?),
            _ => bail!("Invalid Track Model Name {model_name:?}"),
        };

        Ok(Self {
            model_name,
            name,
            builtin_name,
            flags,
            is_creation_track,
            tags,
            sub_tracks,
            keyframes,
            owned_resources,
            animcurve_string,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(self.model_name)?;
        builder.write_gm_string(self.name)?;
        builder.write_enum(self.builtin_name);
        self.flags.serialize(builder)?;
        builder.write_bool32(self.is_creation_track);
        builder.write_usize(self.tags.len())?;
        builder.write_usize(self.owned_resources.len())?;
        builder.write_usize(self.sub_tracks.len())?;
        for tag in &self.tags {
            builder.write_i32(*tag);
        }
        for animation_curve in &self.owned_resources {
            // this can be null if there were no owned resources when parsing
            builder.write_gm_string(self.animcurve_string)?;
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
            Keyframes::Color(k) | Keyframes::Real(k) => k.serialize(builder)?,
            Keyframes::Text(k) => k.serialize(builder)?,
            Keyframes::Particle(k) => k.serialize(builder)?,
            Keyframes::BroadcastMessage(k) => k.serialize(builder)?,
        }
        Ok(())
    }
}

gm_enum!( BuiltinName {
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
});

bitfield_struct! {
    Flags: i32 {
        children_ignore_origin: 0,
    }
}

fn force_read_string<'a>(reader: &mut DataReader<'a>) -> Result<&'a str> {
    let string_pos = reader.read_u32()?;

    let chunk = reader.chunk;
    let pos = reader.cur_pos;
    reader.chunk = reader.string_chunk;
    reader.cur_pos = string_pos - 4;

    let len = reader.read_u32().context("force-reading string length")?;
    let bytes = reader.read_bytes_dyn(len).context("force-reading string")?;
    let string = str::from_utf8(bytes).context_src("force-reading string")?;

    reader.chunk = chunk;
    reader.cur_pos = pos;

    Ok(string)
}
