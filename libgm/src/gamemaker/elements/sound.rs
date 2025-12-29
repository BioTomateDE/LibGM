use macros::named_list_chunk;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMElement, audio_group::GMAudioGroup, embedded_audio::GMEmbeddedAudio},
        reference::GMRef,
        serialize::{builder::DataBuilder, traits::GMSerializeIfVersion},
        version::GMVersion,
    },
    prelude::*,
};

#[named_list_chunk("SOND")]
pub struct GMSounds {
    pub sounds: Vec<GMSound>,
    pub exists: bool,
}

impl GMElement for GMSounds {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let sounds: Vec<GMSound> = reader.read_pointer_list()?;
        Ok(Self { sounds, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_pointer_list(&self.sounds)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMSound {
    /// The name of the sound entry.
    /// This name is used when referencing this entry from code.
    pub name: String,

    /// The flags of this sound.
    /// This field is a bit unstable and may be changed in the future.
    pub flags: Flags,

    /// The file format of the audio entry.
    /// This includes the `.` from the file extension.
    /// Possible values are:
    /// - `.wav`
    /// - `.mp3`
    /// - `.ogg`
    pub audio_type: AudioType,

    /// The original file name of the audio entry.
    /// This is the full filename how it was loaded in the project.
    /// This will be used if the sound effect is streamed from disk to find the sound file.
    ///
    /// This is used if the `Flags.embedded` flag is not set.
    pub file: String,

    /// A pre-`GameMaker Studio` way of having certain effects on a sound effect.
    /// Although the exact way this works is unknown, the following values are possible:
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

    /// The reference to the [`GMEmbeddedAudio`] audio file.
    /// This is used if the `Flags.embedded` flag is set.
    pub audio_file: Option<GMRef<GMEmbeddedAudio>>,

    /// The precomputed length of the sound's audio data.
    /// Introduced in GameMaker 2024.6.
    /// TODO(doc): which unit?
    pub audio_length: Option<f32>,
}

impl GMElement for GMSound {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: String = reader.read_gm_string()?;

        let flags = Flags::deserialize(reader)?;

        let audio_type: Option<String> = reader.read_gm_string_opt()?;
        let audio_type = match audio_type.as_deref() {
            Some("") | None => AudioType::Unknown,
            Some(".wav") => AudioType::Wav,
            Some(".ogg") => AudioType::Ogg,
            Some(".mp3") => AudioType::Mp3,
            Some(other) => {
                let msg = format!("Invalid or unknown audio type {other:?}");
                if reader.options.verify_constants {
                    bail!("{msg}");
                }
                log::warn!("{msg}");
                AudioType::Unknown
            },
        };

        let file: String = reader.read_gm_string()?;
        let effects = reader.read_u32()?;
        let volume = reader.read_f32()?;
        let pitch = reader.read_f32()?;

        let audio_group: GMRef<GMAudioGroup>;
        if reader.general_info.wad_version >= 14 {
            // Only if flags.regular (doesn't exist for now)
            audio_group = reader.read_resource_by_id()?;
        } else {
            let preload = reader.read_bool32().context("reading preload")?;
            reader.assert_bool(preload, true, "Preload")?;
            audio_group = GMRef::new(get_builtin_sound_group_id(&reader.general_info.version));
        }

        let audio_file: Option<GMRef<GMEmbeddedAudio>> = reader.read_resource_by_id_opt()?;
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
            audio_file,
            audio_length,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.name);
        self.flags.serialize(builder)?;

        let audio_type = match self.audio_type {
            AudioType::Unknown => None,
            AudioType::Wav => Some(".wav"),
            AudioType::Ogg => Some(".ogg"),
            AudioType::Mp3 => Some(".mp3"),
        };
        let audio_type = audio_type.map(String::from);

        builder.write_gm_string_opt(&audio_type);
        builder.write_gm_string(&self.file);
        builder.write_u32(self.effects);
        builder.write_f32(self.volume);
        builder.write_f32(self.pitch);
        if builder.wad_version() >= 14 {
            // Only if flags.regular (not stored for now)
            builder.write_resource_id(self.audio_group);
        } else {
            builder.write_bool32(true); // Preload
        }
        builder.write_resource_id_opt(self.audio_file);
        self.audio_length
            .serialize_if_gm_ver(builder, "Audio Length", (2024, 6))?;
        Ok(())
    }
}

#[allow(clippy::bool_to_int_with_if)] // Lol this is a coincidence
/// The exact versions may be inaccurate.
fn get_builtin_sound_group_id(gm_version: &GMVersion) -> u32 {
    // ver >= 1.0.0.1250 || (ver >= 1.0.0.161 && ver < 1.0.0.1000)
    let is_ver = |build| gm_version.is_version_at_least((1, 0, 0, build));
    if is_ver(1250) || is_ver(161) && !is_ver(1000) {
        0
    } else {
        1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioType {
    Unknown,
    Wav,
    Ogg,
    Mp3,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
/// The `regular` flag may be added later to support GM8.
pub struct Flags {
    pub embedded: bool,
    pub compressed: bool,
}

impl GMElement for Flags {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let raw = reader.read_u32()?;
        Ok(match raw {
            100 => Self { embedded: false, compressed: false },
            101 => Self { embedded: true, compressed: false },
            102 => Self { embedded: false, compressed: true },
            103 => Self { embedded: true, compressed: true },
            _ => bail!("Invalid/Unknown sound flags {raw}, please report this error"),
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        let mut raw = 100;
        if self.embedded {
            raw |= 1;
        }
        if self.compressed {
            raw |= 2;
        }
        builder.write_u32(raw);
        Ok(())
    }
}
