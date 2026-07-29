// SPDX-License-Identifier: GPL-3.0-only

use crate::prelude::*;
use crate::wad::chunk::gm_named_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::elem::audio::Audio;
use crate::wad::elem::audio_group::AudioGroup;
use crate::wad::load::reader::DataReader;
use crate::wad::reference::GMRef;
use crate::wad::save::writer::DataWriter;
use crate::wad::version::GMVersion;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Sounds {
    pub elems: Vec<Option<Sound>>,
}

gm_named_list_chunk!(SOND, Sounds, Sound, nullable);

impl GMElement for Sounds {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let elems: Vec<Option<Sound>> = reader.read_pointer_list_opt()?;
        Ok(Self { elems })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_pointer_list_opt(&self.elems)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sound {
    /// The name of the sound entry.
    /// This name is used when referencing this entry from code.
    pub name: GMRef<String>,

    /// The flags of this sound.
    /// This field is a bit unstable and may be changed in the future.
    pub flags: SoundFlags,

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
    /// This is used if the [`SoundFlags::EMBEDDED`] flag is not set.
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

    /// **Unused, usually 0**
    ///
    /// Maybe the pitch change of the audio entry (no idea where this comes from).
    pub pitch: f32,

    /// The audio group this audio entry belongs to.
    /// These can only be used with the regular audio system.
    /// This is used if the [`SoundFlags::REGULAR`] flag is set (always set for now).
    /// For more information, see [`AudioGroup`].
    pub audio_group: GMRef<AudioGroup>,

    /// The reference to the [`Audio`] audio file.
    /// This is used if the [`SoundFlags::EMBEDDED`] flag is set.
    pub audio: GMRef<Audio>,

    /// The precomputed length of the sound's audio data.
    /// Introduced in GameMaker 2024.6.
    /// DOCME: which unit?
    pub audio_length: Option<f32>,
}

impl GMElement for Sound {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let flags = reader.read_u32()?;
        let flags = SoundFlags::from_bits(flags)
            .ok_or_else(|| format!("Invalid Sound Flags {flags:08X}"))?;
        let audio_type: GMRef<String> = reader.read_gm_string()?;
        let file: GMRef<String> = reader.read_gm_string()?;
        let effects = reader.read_u32()?;
        let volume = reader.read_f32()?;
        let pitch = reader.read_f32()?;

        let agrp = reader.version >= GMVersion::Wad14 && flags.contains(SoundFlags::REGULAR);
        let audio_group: GMRef<AudioGroup> = if agrp {
            reader.read_resource_by_id()?
        } else {
            let preload = reader.read_bool32().ctx("reading preload")?;
            reader.assert_bool(preload, true, "Preload")?;
            GMRef::none()
        };

        let audio: GMRef<Audio> = reader.read_resource_by_id()?;
        let audio_length: Option<f32> = reader.deserialize_if_version(GMVersion::GM2024_6)?;

        // // GMS1 asset compiler is stupid and keeps repeating the previous embedded audio id
        // // if the current sound is external.
        // if !flags.contains(SoundFlags::EMBEDDED) {
        //     audio = GMRef::none();
        // }

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

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_gm_string(self.name)?;
        writer.write_u32(self.flags.bits());
        writer.write_gm_string(self.audio_type)?;
        writer.write_gm_string(self.file)?;
        writer.write_u32(self.effects);
        writer.write_f32(self.volume);
        writer.write_f32(self.pitch);
        if writer.version() >= GMVersion::Wad14 && self.flags.contains(SoundFlags::REGULAR) {
            writer.write_resource_id(self.audio_group);
        } else {
            writer.write_bool32(true); // Preload
        }
        writer.write_resource_id(self.audio);
        writer.write_if_ver(&self.audio_length, "Audio Length", GMVersion::GM2024_6)?;
        Ok(())
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SoundFlags: u32 {
        const EMBEDDED = 0x1;
        const COMPRESSED = 0x2;
        const REGULAR = 0x64;
    }
}
