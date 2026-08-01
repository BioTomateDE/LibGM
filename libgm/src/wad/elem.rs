// SPDX-License-Identifier: GPL-3.0-only
//! This is THE module. All GameMaker elements are contained in these submodules.
//! There are some traits as well which can help you with dynamic programming.

use std::collections::HashMap;

use crate::prelude::*;
use crate::util::fmt::typename;
use crate::wad::elem::string::Strings;
use crate::wad::load::reader::DataReader;
use crate::wad::reference::GMRef;
use crate::wad::save::writer::DataWriter;

pub mod animation_curve;
pub mod audio;
pub mod audio_group;
pub mod code;
pub(super) mod data_file;
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
pub mod tileset;
pub mod timeline;
pub mod ui_node;
pub mod variable;

pub(crate) trait GMComplexElement: Sized {
    const PRE_PADDING: u32 = 1;
    const POST_PADDING: u32 = 1;
    type ReadContext: Copy;
    type WriteContext: Copy;

    /// Deserializes this element from the current position of the reader.
    ///
    /// Implementations should read the exact binary representation of this
    /// element and return a fully constructed instance.
    fn deserialize_complex(reader: &mut DataReader, context: Self::ReadContext) -> Result<Self>;

    /// Serializes this element to the current position of the writer.
    ///
    /// Implementations should write the exact binary representation of this
    /// element in the format expected by the GameMaker runtime.
    fn serialize_complex(&self, writer: &mut DataWriter, context: Self::WriteContext)
    -> Result<()>;
}

pub(crate) trait GMElement: Sized {
    fn deserialize(reader: &mut DataReader) -> Result<Self>;
    fn serialize(&self, writer: &mut DataWriter) -> Result<()>;
}

impl<T: GMElement> GMComplexElement for T {
    type ReadContext = ();
    type WriteContext = ();

    fn deserialize_complex(reader: &mut DataReader, _context: ()) -> Result<Self> {
        Self::deserialize(reader)
    }

    fn serialize_complex(&self, writer: &mut DataWriter, _context: ()) -> Result<()> {
        self.serialize(writer)
    }
}

impl GMElement for u8 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_u8()
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_u8(*self);
        Ok(())
    }
}

impl GMElement for i8 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_i8()
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_i8(*self);
        Ok(())
    }
}

impl GMElement for u16 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_u16()
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_u16(*self);
        Ok(())
    }
}

impl GMElement for i16 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_i16()
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_i16(*self);
        Ok(())
    }
}

impl GMElement for u32 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_u32()
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_u32(*self);
        Ok(())
    }
}

impl GMElement for i32 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_i32()
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_i32(*self);
        Ok(())
    }
}

impl GMElement for u64 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_u64()
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_u64(*self);
        Ok(())
    }
}

impl GMElement for i64 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_i64()
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_i64(*self);
        Ok(())
    }
}

impl GMElement for f32 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_f32()
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_f32(*self);
        Ok(())
    }
}

impl GMElement for f64 {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_f64()
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_f64(*self);
        Ok(())
    }
}

impl GMElement for bool {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_bool32()
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_bool32(*self);
        Ok(())
    }
}

// TODO: this also allows texture page items. should there be a GMAsset trait or something?
impl<T: GMComplexElement> GMElement for GMRef<T> {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_resource_by_id()
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_resource_id(*self);
        Ok(())
    }
}

impl GMElement for GMRef<String> {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        reader.read_gm_string()
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_gm_string(*self)
    }
}

/// Validates all names of the root elements in this chunk.
///
/// This checks for duplicates as well as names not following the proper charset.
pub(crate) fn validate_names<T: GMAssetListChunk>(chunk: &T, gm_strings: &Strings) -> Result<()> {
    // PERF: this can probably be optimised or something
    let mut seen: HashMap<&String, GMRef<_>> = HashMap::new();

    for (i, item) in chunk.element_refs() {
        let name = item.name(gm_strings)?;

        item.validate_name(gm_strings)
            .ctx(|| format!("validating {} with name {:?}", typename::<T::Element>(), name,))?;

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
pub trait GMNamedElement: GMComplexElement {
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
