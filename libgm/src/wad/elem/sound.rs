// SPDX-License-Identifier: GPL-3.0-only

use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::gm_named_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::elem::audio::GMAudio;
use crate::wad::elem::audio_group::GMAudioGroup;
use crate::wad::parse::reader::DataReader;
use crate::wad::reference::GMRef;
use crate::wad::version::GMVersion;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GMSounds {
    pub elems: Vec<Option<GMSound>>,
    pub exists: bool,
}

gm_named_list_chunk!(SOND, GMSounds, GMSound, nullable);

impl GMElement for GMSounds {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let elems: Vec<Option<GMSound>> = reader.read_pointer_list_opt()?;
        Ok(Self { elems, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_pointer_list_opt(&self.elems)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMSound {
    /// The name of the sound entry.
    /// This name is used when referencing this entry from code.
    pub name: GMRef<String>,

    /// The flags of this sound.
    /// This field is a bit unstable and may be changed in the future.
    pub flags: Flags,

    /// The file format of the audio entry.
    /// This includes the `.` from the file extension.
    /// Possible values are:
    /// - `.wav`
    /// - `.mp3`
    /// - `.ogg`
    pub audio_type: GMRef<String>,

    /// The original file name of the audio entry.
    /// This is the full filename how it was loaded in the project.
    /// This will be used if the sound effect is streamed from disk to find the
    /// sound file.
    ///
    /// This is used if the `Flags.embedded` flag is not set.
    pub file: GMRef<String>,

    /// A pre-`GameMaker Studio` way of having certain effects on a sound
    /// effect. Although the exact way this works is unknown, the following
    /// values are possible:
    /// - Chorus
    /// - Echo
    /// - Flanger
    /// - Gargle
    /// - Reverb
    ///
    /// These can be combined with each other, apparently.
    /// Discussion here: <https://discord.com/channels/566861759210586112/568950566122946580/957318910066196500>
    pub effects: u32,

    /// The volume the audio entry is played at.
    ///
    /// Valid Range: `0.0` - `1.0`.
    pub volume: f32,

    /// The pitch change of the audio entry (maybe? more research needed).
    pub pitch: f32,

    /// The audio group this audio entry belongs to.
    /// These can only be used with the regular audio system.
    /// This is used if the `Flags.regular` flag is set (always set for now).
    /// For more information, see [`GMAudioGroup`].
    pub audio_group: GMRef<GMAudioGroup>,

    /// The reference to the [`GMAudio`] audio file.
    /// This is used if the `Flags.embedded` flag is set.
    pub audio: GMRef<GMAudio>,

    /// The precomputed length of the sound's audio data.
    /// Introduced in GameMaker 2024.6.
    /// TODO(doc): which unit?
    pub audio_length: Option<f32>,
}

impl GMElement for GMSound {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let flags = reader.read_u32()?;
        let flags =
            Flags::from_bits(flags).ok_or_else(|| format!("Invalid Sound Flags {flags:08X}"))?;
        let audio_type: GMRef<String> = reader.read_gm_string()?;
        let file: GMRef<String> = reader.read_gm_string()?;
        let effects = reader.read_u32()?;
        let volume = reader.read_f32()?;
        let pitch = reader.read_f32()?;

        let audio_group: GMRef<GMAudioGroup> =
            if reader.general_info.wad_version >= 14 && flags.contains(Flags::REGULAR) {
                reader.read_resource_by_id()?
            } else {
                let preload = reader.read_bool32().ctx("reading preload")?;
                reader.assert_bool(preload, true, "Preload")?;
                GMRef::new(get_builtin_sound_group_id(reader.general_info.version))
            };

        let audio: GMRef<GMAudio> = reader.read_resource_by_id()?;
        let audio_length: Option<f32> = reader.deserialize_if_gm_version((2024, 6))?;

        Ok(Self {
            name,
            flags,
            audio_type,
            file,
            effects,
            volume,
            pitch,
            audio_group,
            audio,
            audio_length,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(self.name)?;
        builder.write_u32(self.flags.bits());
        builder.write_gm_string(self.audio_type)?;
        builder.write_gm_string(self.file)?;
        builder.write_u32(self.effects);
        builder.write_f32(self.volume);
        builder.write_f32(self.pitch);
        if builder.wad_version() >= 14 && self.flags.contains(Flags::REGULAR) {
            builder.write_resource_id(self.audio_group);
        } else {
            builder.write_bool32(true); // Preload
        }
        builder.write_resource_id(self.audio);
        builder.write_if_ver(&self.audio_length, "Audio Length", (2024, 6))?;
        Ok(())
    }
}

#[allow(clippy::bool_to_int_with_if)] // lol this is a coincidence
/// The exact versions may be inaccurate.
fn get_builtin_sound_group_id(version: GMVersion) -> i32 {
    if test_gms1_version(version, 1250, 161) {
        0
    } else {
        1
    }
}

// This function could be reused for other elements.
#[must_use]
fn test_gms1_version(version: GMVersion, stable_build: u32, beta_build: u32) -> bool {
    assert!(beta_build < 1000);
    version > (1, 0, 0, stable_build) || version > (1, 0, 0, beta_build)
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Flags: u32 {
        const EMBEDDED = 0x1;
        const COMPRESSED = 0x2;
        const REGULAR = 0x64;
    }
}
