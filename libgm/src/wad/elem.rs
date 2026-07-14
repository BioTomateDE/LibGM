// SPDX-License-Identifier: GPL-3.0-only
//! This is THE module. All GameMaker elements are contained in these submodules.
//! There are some traits as well which can help you with dynamic programming.

use std::collections::HashMap;

use crate::prelude::*;
use crate::util::fmt::typename;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::string::Strings;
use crate::wad::parse::reader::DataReader;
use crate::wad::reference::GMRef;

pub mod animation_curve;
pub mod audio;
pub mod audio_group;
pub mod background;
pub mod code;
pub mod data_file;
pub mod embedded_image;
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
pub mod string;
pub mod tag;
pub mod texture_group_info;
pub mod texture_page;
pub mod texture_page_item;
pub mod timeline;
pub mod ui_node;
pub mod variable;

/// All GameMaker elements that can be deserialized
/// from a data file should implement this trait.
#[expect(unused_variables)]
pub(crate) trait GMElement: Sized {
    /// Deserializes this element from the current position of the reader.
    ///
    /// Implementations should read the exact binary representation of this
    /// element and return a fully constructed instance.
    fn deserialize(reader: &mut DataReader) -> Result<Self>;

    /// Serializes this element to the current position of the builder.
    ///
    /// Implementations should write the exact binary representation of this
    /// element in the format expected by the GameMaker runtime.
    fn serialize(&self, builder: &mut DataBuilder) -> Result<()>;

    /// Handles padding bytes that may appear before this element in pointer lists.
    ///
    /// This is called before [`GMElement::deserialize`] when reading from
    /// structured data. The default implementation does nothing - override
    /// if your element requires alignment padding in specific contexts.
    fn deserialize_pre_padding(reader: &mut DataReader) -> Result<()> {
        Ok(())
    }

    /// Writes padding bytes that may be required before this element in pointer lists.
    ///
    /// This is called before [`GMElement::serialize`] when writing to
    /// structured data. The default implementation does nothing - override
    /// if your element requires alignment padding in specific contexts.
    fn serialize_pre_padding(&self, builder: &mut DataBuilder) -> Result<()> {
        Ok(())
    }

    /// Handles padding bytes that may appear after this element in pointer lists.
    ///
    /// This is called after [`GMElement::deserialize`] when reading from
    /// structured data. The `is_last` parameter indicates if this is the
    /// final element in a list, which may affect padding requirements.
    fn deserialize_post_padding(reader: &mut DataReader, is_last: bool) -> Result<()> {
        Ok(())
    }

    /// Writes padding bytes that may be required after this element in pointer lists.
    ///
    /// This is called after [`GMElement::serialize`] when writing to structured
    /// data. The `is_last` parameter indicates if this is the final element
    /// in a list, which may affect padding requirements.
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

// TODO: this also allows texture page items. should there be a GMAsset trait or something?
impl<T: GMElement> GMElement for GMRef<T> {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_resource_by_id()
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_resource_id(*self);
        Ok(())
    }
}

impl GMElement for GMRef<String> {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_gm_string()
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(*self)
    }
}

/// Validates all names of the root elements in this chunk.
///
/// This checks for duplicates as well as names not following the proper charset.
pub(crate) fn validate_names<T: GMNamedListChunk>(chunk: &T, gm_strings: &Strings) -> Result<()> {
    // PERF: this can probably be optimised or something
    let mut seen: HashMap<&String, GMRef<_>> = HashMap::new();

    for (i, item) in chunk.element_refs() {
        let name = item.name(gm_strings)?;

        item.validate_name(gm_strings).ctx(|| {
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
            "There are multiple {} with the same name ({:?}): First at {:?} and now at {:?}",
            typename::<T::Element>(),
            name,
            first_index,
            i,
        );
    }
    Ok(())
}

/// All GameMaker elements with a unique name (to the list
/// they're contained in) should implement this trait.
#[expect(private_bounds)]
// TODO: maybe turn this into a `GMAsset` trait that only applies to thing referencable by name in gml
pub trait GMNamedElement: GMElement {
    /// The name of this element as a `GMRef<String>`.
    #[must_use]
    fn name_ref(&self) -> GMRef<String>;

    /// The name of this element as a `&String`.
    fn name<'a>(&self, gm_strings: &'a Strings) -> Result<&'a String> {
        self.name_ref().resolve(&gm_strings.elems)
    }

    /// Whether the name of this element is valid.
    /// This method respects this element type's specific rules.
    fn validate_name(&self, gm_strings: &Strings) -> Result<()> {
        validate_identifier(self.name(gm_strings)?)
    }
}

/// Generic check whether an identifier / asset name is valid.
/// Some element types might have different rules.
/// These should be defined in [`GMNamedElement::validate_name`].
///
/// ## Rules:
/// - At least one character long
/// - Ascii letters, digits and underscores are allowed (`@` is also allowed
///   because GameMaker uses them internally)
/// - First character is not a digit
pub(crate) fn validate_identifier(name: &str) -> Result<()> {
    let first_char = name.chars().next().ok_or("Identifier is empty")?;

    if first_char.is_ascii_digit() {
        bail!("Identifier {name:?} starts with a digit ({first_char})");
    }

    for ch in name.chars() {
        // @ is used by GameMaker internally.
        if !matches!(ch, 'a'..='z'| '0'..='9' | '_' | 'A'..='Z' | '@') {
            bail!("Identifier {name:?} contains invalid character {ch:?}");
        }
    }

    Ok(())
}

macro_rules! element_stub {
    ($type:ty) => {
        impl $crate::wad::elem::GMElement for $type {
            fn deserialize(_: &mut $crate::wad::parse::reader::DataReader) -> Result<Self> {
                unimplemented!(
                    "Using {0}::deserialize is not supported, use {0}s::deserialize instead",
                    stringify!($type),
                );
            }

            fn serialize(&self, _: &mut $crate::wad::build::builder::DataBuilder) -> Result<()> {
                unimplemented!(
                    "Using {0}::serialize is not supported, use {0}s::serialize instead",
                    stringify!($type),
                );
            }
        }
    };
}

pub(crate) use element_stub;
