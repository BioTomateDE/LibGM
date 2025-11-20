use crate::gamemaker::deserialize::reader::DataReader;
use crate::gamemaker::deserialize::resources::GMRef;
use crate::gamemaker::elements::audio_groups::GMAudioGroup;
use crate::gamemaker::elements::embedded_audio::GMEmbeddedAudio;
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::gm_version::GMVersion;
use crate::gamemaker::serialize::builder::DataBuilder;
use crate::gamemaker::serialize::traits::GMSerializeIfVersion;
use crate::prelude::*;
use crate::util::assert::assert_bool;
use crate::util::bitfield::bitfield_struct;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Default)]
pub struct GMSounds {
    pub sounds: Vec<GMSound>,
    pub exists: bool,
}

impl Deref for GMSounds {
    type Target = Vec<GMSound>;
    fn deref(&self) -> &Self::Target {
        &self.sounds
    }
}

impl DerefMut for GMSounds {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sounds
    }
}

impl GMChunkElement for GMSounds {
    const NAME: &'static str = "SOND";
    fn exists(&self) -> bool {
        self.exists
    }
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

    /// The flags the sound entry uses.
    /// These effectively control different options of this sound.
    ///
    /// For more information, see [`GMSoundFlags`].
    pub flags: GMSoundFlags,

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
        let flags = GMSoundFlags::deserialize(reader)?;
        let audio_type: Option<String> = reader.read_gm_string_opt()?;
        let file: String = reader.read_gm_string()?;
        let effects = reader.read_u32()?;
        let volume = reader.read_f32()?;
        let pitch = reader.read_f32()?;
        let audio_group: GMRef<GMAudioGroup>;

        if flags.regular && reader.general_info.bytecode_version >= 14 {
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
        builder.write_gm_string_opt(&self.audio_type);
        builder.write_gm_string(&self.file);
        builder.write_u32(self.effects);
        builder.write_f32(self.volume);
        builder.write_f32(self.pitch);
        if self.flags.regular && builder.bytecode_version() >= 14 {
            builder.write_resource_id(self.audio_group);
        } else {
            builder.write_bool32(true); // Preload
        }
        builder.write_resource_id_opt(&self.audio_file);
        self.audio_length
            .serialize_if_gm_ver(builder, "Audio Length", (2024, 6))?;
        Ok(())
    }
}

bitfield_struct! {
    /// Audio entry flags a sound entry can use.
    GMSoundFlags : u32 {
        /// Whether the sound is embedded into the data file.
        /// This should ideally be used for sound effects, but not for music.
        /// The `GameMaker` documentation also calls this "not streamed"
        /// (or "from memory") for when the flag is present, or "streamed" when it isn't.
        is_embedded: 0,

        /// Whether the sound is compressed.
        /// When a sound is compressed it will take smaller memory/disk space.
        /// However, this is at the cost of needing to decompress it when it needs to be played,
        /// which means slightly higher CPU usage.
        ///
        /// TODO: how is it compressed? and when is this flag even set
        is_compressed: 1,

        /// Whether this sound uses the "new audio system".
        /// This is default for everything post `GameMaker Studio`.
        /// The legacy sound system was used in pre `GameMaker 8`.
        regular: 6,
    }
}

fn get_builtin_sound_group_id(gm_version: &GMVersion) -> u32 {
    let is_ver = |req| gm_version.is_version_at_least(req); // Small closure for concision
    // ver >= 1.0.0.1250 || (ver >= 1.0.0.161 && ver < 1.0.0.1000)
    if is_ver((1, 0, 0, 1250)) || is_ver((1, 0, 0, 161)) && !is_ver((1, 0, 0, 1000)) {
        0
    } else {
        1
    }
}
