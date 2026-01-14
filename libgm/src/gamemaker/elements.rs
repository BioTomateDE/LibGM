//! This is THE module. All GameMaker elements are contained in these submodules.
//! There are some traits as well which can help you with dynamic programming.

use std::collections::HashMap;

use crate::{
    gamemaker::{
        chunk::ChunkName, deserialize::reader::DataReader, reference::GMRef,
        serialize::builder::DataBuilder,
    },
    prelude::*,
    util::fmt::typename,
};

pub mod animation_curve;
pub mod audio_group;
pub mod background;
pub mod code;
pub(crate) mod data_file;
pub mod embedded_audio;
pub mod embedded_image;
pub mod embedded_texture;
pub mod extension;
pub mod feature_flag;
pub mod filter_effect;
pub mod font;
pub mod function;
pub mod game_end;
pub mod game_object;
pub mod general_info;
pub mod global_init;
pub mod language;
pub mod options;
pub mod particle_emitter;
pub mod particle_system;
pub mod path;
pub mod room;
pub mod script;
pub mod sequence;
pub mod shader;
pub mod sound;
pub mod sprite;
pub(crate) mod string;
pub mod tag;
pub mod texture_group_info;
pub mod texture_page_item;
pub mod timeline;
pub mod ui_node;
pub mod variable;

/// All GameMaker elements that can be deserialized
/// from a data file should implement this trait.
#[allow(unused_variables)]
pub trait GMElement: Sized {
    /// Deserializes this element from the current position of the reader.
    ///
    /// Implementations should read the exact binary representation of this element
    /// and return a fully constructed instance.
    fn deserialize(reader: &mut DataReader) -> Result<Self>;

    /// Serializes this element to the current position of the builder.
    ///
    /// Implementations should write the exact binary representation of this element
    /// in the format expected by the GameMaker runtime.
    fn serialize(&self, builder: &mut DataBuilder) -> Result<()>;

    /// Handles padding bytes that may appear before this element in pointer lists.
    ///
    /// This is called before [`GMElement::deserialize`] when reading from structured data.
    /// The default implementation does nothing - override if your element requires
    /// alignment padding in specific contexts.
    fn deserialize_pre_padding(reader: &mut DataReader) -> Result<()> {
        Ok(())
    }

    /// Writes padding bytes that may be required before this element in pointer lists.
    ///
    /// This is called before [`GMElement::serialize`] when writing to structured data.
    /// The default implementation does nothing - override if your element requires
    /// alignment padding in specific contexts.
    fn serialize_pre_padding(&self, builder: &mut DataBuilder) -> Result<()> {
        Ok(())
    }

    /// Handles padding bytes that may appear after this element in pointer lists.
    ///
    /// This is called after [`GMElement::deserialize`] when reading from structured data.
    /// The `is_last` parameter indicates if this is the final element in a list,
    /// which may affect padding requirements.
    fn deserialize_post_padding(reader: &mut DataReader, is_last: bool) -> Result<()> {
        Ok(())
    }

    /// Writes padding bytes that may be required after this element in pointer lists.
    ///
    /// This is called after [`GMElement::serialize`] when writing to structured data.
    /// The `is_last` parameter indicates if this is the final element in a list,
    /// which may affect padding requirements.
    fn serialize_post_padding(&self, builder: &mut DataBuilder, is_last: bool) -> Result<()> {
        Ok(())
    }
}

impl GMElement for u8 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_u8()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_u8(*self);
        Ok(())
    }
}

impl GMElement for i8 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_i8()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i8(*self);
        Ok(())
    }
}

impl GMElement for u16 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_u16()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_u16(*self);
        Ok(())
    }
}

impl GMElement for i16 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_i16()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i16(*self);
        Ok(())
    }
}

impl GMElement for u32 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_u32()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_u32(*self);
        Ok(())
    }
}

impl GMElement for i32 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_i32()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(*self);
        Ok(())
    }
}

impl GMElement for u64 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_u64()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_u64(*self);
        Ok(())
    }
}

impl GMElement for i64 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_i64()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i64(*self);
        Ok(())
    }
}

impl GMElement for f32 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_f32()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_f32(*self);
        Ok(())
    }
}

impl GMElement for f64 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_f64()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_f64(*self);
        Ok(())
    }
}

impl GMElement for bool {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_bool32()
    }
    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_bool32(*self);
        Ok(())
    }
}

impl GMElement for String {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_gm_string()
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(self);
        Ok(())
    }
}

impl GMElement for Option<String> {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_gm_string_opt()
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string_opt(self);
        Ok(())
    }
}

impl<T> GMElement for GMRef<T> {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_resource_by_id()
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_resource_id(*self);
        Ok(())
    }
}

impl<T> GMElement for Option<GMRef<T>> {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_resource_by_id_opt()
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_resource_id_opt(*self);
        Ok(())
    }
}

/// All chunk elements should implement this trait.
pub trait GMChunk: GMElement + Default {
    /// The four character GameMaker chunk name (GEN8, STRG, VARI, etc.).
    const NAME: ChunkName;

    /// Returns `true` if this chunk is present in the data file.
    ///
    /// This differs from simply checking if the chunk is empty:
    /// - A list chunk may exist and contain zero elements.
    ///   > Chunk name + chunk length (four) + element count (zero).
    /// - A chunk may exist but contain no data.
    ///   > Chunk name + chunk length (zero).
    /// - A chunk may be absent entirely from the file format.
    ///   > Completely gone.
    ///
    /// Use this to distinguish between "present but empty" and "not present at all".
    fn exists(&self) -> bool;
}

/// All chunk elements that represent a collection of elements should implement this trait.
/// The only exceptions are `GEN8` and `OPTN`.
pub trait GMListChunk: GMChunk {
    type Element: GMElement;

    #[must_use]
    fn elements(&self) -> &Vec<Self::Element>;

    #[must_use]
    fn elements_mut(&mut self) -> &mut Vec<Self::Element>;

    fn by_ref(&self, gm_ref: GMRef<Self::Element>) -> Result<&Self::Element> {
        gm_ref.resolve(self.elements())
    }

    fn by_ref_mut(&mut self, gm_ref: GMRef<Self::Element>) -> Result<&mut Self::Element> {
        gm_ref.resolve_mut(self.elements_mut())
    }

    fn push(&mut self, element: Self::Element) {
        self.elements_mut().push(element);
    }

    #[must_use]
    fn len(&self) -> usize {
        self.elements().len()
    }

    #[must_use]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn iter(&self) -> core::slice::Iter<'_, Self::Element>;
    fn iter_mut(&mut self) -> core::slice::IterMut<'_, Self::Element>;
    fn into_iter(self) -> std::vec::IntoIter<Self::Element>;
}

/// All chunk elements that represent a collection of elements **with a unique name** should implement this trait.
pub trait GMNamedListChunk: GMListChunk<Element: GMNamedElement> {
    fn ref_by_name(&self, name: &str) -> Result<GMRef<Self::Element>>;
    fn by_name(&self, name: &str) -> Result<&Self::Element>;
    fn by_name_mut(&mut self, name: &str) -> Result<&mut Self::Element>;
}

/// Validates all names of the root elements in this chunk.
///
/// This checks for duplicates as well as names not following the proper charset.
pub fn validate_names<T: GMNamedListChunk>(chunk: &T) -> Result<()> {
    // TODO(perf): this can probably be optimised or something
    let elements = chunk.elements();
    let mut seen: HashMap<&String, usize> = HashMap::new();

    for (i, item) in elements.iter().enumerate() {
        let name = item.name();

        item.validate_name().with_context(|| {
            format!(
                "validating {} with name {:?}",
                typename::<T::Element>(),
                name,
            )
        })?;

        let Some(first_index) = seen.insert(name, i) else {
            continue;
        };

        bail!(
            "There are multiple {} with the same name ({:?}): \
            First at index {} and now at index {}",
            typename::<T::Element>(),
            name,
            first_index,
            i,
        );
    }
    Ok(())
}

#[allow(unused_variables)]
/// All GameMaker elements with a unique name (to the list they're contained in) should implement this trait.
pub trait GMNamedElement: GMElement {
    /// The name of this element.
    #[must_use]
    fn name(&self) -> &String;

    /// A mutable reference to the name of this element.
    #[must_use]
    fn name_mut(&mut self) -> &mut String;

    /// Whether the name of this element is valid.
    /// This method respects this element type's specific rules.
    fn validate_name(&self) -> Result<()> {
        validate_identifier(self.name())
    }
}

/// Generic check whether an identifier / asset name is valid.
/// Some element types might have different rules.
/// These should be defined in [`GMNamedElement::validate_name`].
///
/// ## Rules:
/// - At least one character long
/// - First character is not a digit
/// - Letters and underscores are allowed
/// - Only ascii characters
/// - Special `@@MyIdentifier@@` syntax covered
pub(crate) fn validate_identifier(name: &str) -> Result<()> {
    let orig_name = name;
    let mut name = name;

    // i hate this function
    if name.len() > 4 && name.starts_with("@@") && name.ends_with("@@") {
        name = &name[2..name.len() - 2];
    }

    let mut chars = name.chars();
    let first_char = chars.next().ok_or("Identifier is empty")?;

    if !matches!(first_char, 'a'..='z' | '_' | 'A'..='Z') {
        if first_char.is_ascii_digit() {
            bail!("Identifier {orig_name:?} starts with a digit ({first_char})");
        }
        bail!("Identifier {orig_name:?} starts with invalid character {first_char:?}");
    }

    for ch in chars {
        if !matches!(ch, 'a'..='z'| '0'..='9' | '_' | 'A'..='Z') {
            bail!("Identifier {orig_name:?} contains invalid character {ch:?}");
        }
    }

    Ok(())
}

macro_rules! element_stub {
    ($type:ty) => {
        impl $crate::gamemaker::elements::GMElement for $type {
            fn deserialize(
                _: &mut $crate::gamemaker::deserialize::reader::DataReader,
            ) -> Result<Self> {
                unimplemented!(
                    "Using {0}::deserialize is not supported, use {0}s::deserialize instead",
                    stringify!($type),
                );
            }

            fn serialize(
                &self,
                _: &mut $crate::gamemaker::serialize::builder::DataBuilder,
            ) -> Result<()> {
                unimplemented!(
                    "Using {0}::serialize is not supported, use {0}s::serialize instead",
                    stringify!($type),
                );
            }
        }
    };
}

pub(crate) use element_stub;
