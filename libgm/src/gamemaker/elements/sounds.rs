use macros::list_chunk;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{
            GMChunkElement, GMElement, audio_groups::GMAudioGroup, embedded_audio::GMEmbeddedAudio,
        },
        gm_version::GMVersion,
        reference::GMRef,
        serialize::{builder::DataBuilder, traits::GMSerializeIfVersion},
    },
    prelude::*,
    util::assert::assert_bool,
};

#[list_chunk("SOND")]
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

    /// The raw flags of this sound.
    /// WARNING: This field is unstable and may be removed inthe future.
    pub flags: u32,

    /// Whether this sound uses the new audio system (post GM8).
    pub flag_regular: bool,

    /// The file format of the audio entry.
    /// This includes the `.` from the file extension.
    /// Possible values are:
    /// - `.wav`
    /// - `.mp3`
    /// - `.ogg`
    pub audio_type: Option<String>,

    /// The original file name of the audio entry.
    /// This is the full filename how it was loaded in the project.
    /// This will be used if the sound effect is streamed from disk to find the sound file.
    ///
    /// This is used if the [`GMSoundFlags::is_embedded`] flag is set.
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
    /// This is used if the [`GMSoundFlags::regular`] flag is set.
    /// For more information, see [`GMAudioGroup`].
    pub audio_group: GMRef<GMAudioGroup>,

    /// The reference to the `[GMEmbeddedAudio]` audio file.
    /// This is used if the [`GMSoundFlags::is_embedded`] flag is set.
    pub audio_file: Option<GMRef<GMEmbeddedAudio>>,

    /// The precomputeed length of the sound's audio data.
    /// Introduced in `GameMaker` 2024.6.
    /// TODO: which unit
    pub audio_length: Option<f32>,
}

impl GMElement for GMSound {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: String = reader.read_gm_string()?;

        let flags = reader.read_u32()?;
        let flag_regular = (flags >> 5) & 1 == 1;

        let audio_type: Option<String> = reader.read_gm_string_opt()?;
        let file: String = reader.read_gm_string()?;
        let effects = reader.read_u32()?;
        let volume = reader.read_f32()?;
        let pitch = reader.read_f32()?;

        let audio_group: GMRef<GMAudioGroup>;
        if flag_regular && reader.general_info.bytecode_version >= 14 {
            audio_group = reader.read_resource_by_id()?;
        } else {
            let preload = reader.read_bool32()?;
            assert_bool("Preload", true, preload)?;
            audio_group = GMRef::new(get_builtin_sound_group_id(&reader.general_info.version));
        }

        let audio_file: Option<GMRef<GMEmbeddedAudio>> = reader.read_resource_by_id_opt()?;
        let audio_length: Option<f32> = reader.deserialize_if_gm_version((2024, 6))?;

        Ok(Self {
            name,
            flags,
            flag_regular,
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
        builder.write_u32(self.flags);
        builder.write_gm_string_opt(&self.audio_type);
        builder.write_gm_string(&self.file);
        builder.write_u32(self.effects);
        builder.write_f32(self.volume);
        builder.write_f32(self.pitch);
        if self.flag_regular && builder.bytecode_version() >= 14 {
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

fn get_builtin_sound_group_id(gm_version: &GMVersion) -> u32 {
    let is_ver = |req| gm_version.is_version_at_least(req); // Small closure for concision
    // ver >= 1.0.0.1250 || (ver >= 1.0.0.161 && ver < 1.0.0.1000)
    u32::from(!(is_ver((1, 0, 0, 1250)) || is_ver((1, 0, 0, 161)) && !is_ver((1, 0, 0, 1000))))
}
