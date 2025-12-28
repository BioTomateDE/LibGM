mod audio;
mod bool;
mod broadcast;
pub mod color;
mod graphic;
mod instance;
mod moment;
mod particle;
mod sequence;
mod sprite_frames;
mod string;
mod text;

pub use audio::Audio;
pub use bool::Bool;
pub use broadcast::BroadcastMessage;
pub use color::Color;
pub use graphic::Graphic;
pub use instance::Instance;
pub use moment::Moment;
pub use particle::Particle;
pub use sequence::Sequence;
pub use sprite_frames::SpriteFrames;
pub use string::String;
pub use text::Text;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
    util::fmt::typename,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Keyframes {
    Audio(Data<Audio>),
    Instance(Data<Instance>),
    Graphic(Data<Graphic>),
    Sequence(Data<Sequence>),
    SpriteFrames(Data<SpriteFrames>),
    Bool(Data<Bool>),
    // Asset(Data<Asset>),
    String(Data<String>),
    // Int(Data<Int>),
    Color(color::KeyframesData),
    Real(color::KeyframesData),
    Text(Data<Text>),
    Particle(Data<Particle>),
    BroadcastMessage(Data<BroadcastMessage>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Data<T: GMElement> {
    pub keyframes: Vec<Keyframe<T>>,
}

impl<T: GMElement> GMElement for Data<T> {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.align(4)?;
        let keyframes: Vec<Keyframe<T>> = reader
            .read_simple_list()
            .with_context(|| format!("deserializing {} keyframes", typename::<T>()))?;
        Ok(Self { keyframes })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.align(4);
        builder
            .write_simple_list(&self.keyframes)
            .with_context(|| format!("serializing {} keyframes", typename::<T>()))?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Keyframe<T: GMElement> {
    pub key: f32,
    pub length: f32,
    pub stretch: bool,
    pub disabled: bool,
    pub channels: Vec<Channel<T>>,
}
impl<T: GMElement> GMElement for Keyframe<T> {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let key = reader.read_f32()?;
        let length = reader.read_f32()?;
        let stretch = reader.read_bool32()?;
        let disabled = reader.read_bool32()?;
        let channels: Vec<Channel<T>> = reader.read_simple_list()?;
        Ok(Self { key, length, stretch, disabled, channels })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_f32(self.key);
        builder.write_f32(self.length);
        builder.write_bool32(self.stretch);
        builder.write_bool32(self.disabled);
        builder.write_simple_list(&self.channels)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Channel<T: GMElement> {
    pub id: i32,
    pub value: T,
}

impl<T: GMElement> GMElement for Channel<T> {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let id = reader.read_i32()?;
        let value = T::deserialize(reader)
            .with_context(|| format!("deserializing {} channel", typename::<T>()))?;
        Ok(Self { id, value })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(self.id);
        self.value
            .serialize(builder)
            .with_context(|| format!("serializing {} channel", typename::<T>()))?;
        Ok(())
    }
}
