// SPDX-License-Identifier: GPL-3.0-only
use std::fmt::Display;
use std::fmt::Formatter;

use crate::prelude::*;
use crate::util::fmt::hexdump;
use crate::wad::elem::GMElement;
use crate::wad::elem::string::Strings;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChunkName {
    ACRV,
    AGRP,
    AUDO,
    BGND,
    CODE,
    DAFL,
    EMBI,
    EXTN,
    FEAT,
    FEDS,
    FONT,
    FUNC,
    GEN8,
    GLOB,
    GMEN,
    LANG,
    OBJT,
    OPTN,
    PATH,
    PSEM,
    PSYS,
    ROOM,
    SCPT,
    SEQN,
    SHDR,
    SOND,
    SPRT,
    STRG,
    TAGS,
    TGIN,
    TMLN,
    TPAG,
    TXTR,
    UILR,
    VARI,
}

impl ChunkName {
    pub fn from_bytes(bytes: [u8; 4]) -> Result<Self> {
        Ok(match &bytes {
            b"ACRV" => Self::ACRV,
            b"AGRP" => Self::AGRP,
            b"AUDO" => Self::AUDO,
            b"BGND" => Self::BGND,
            b"CODE" => Self::CODE,
            b"DAFL" => Self::DAFL,
            b"EMBI" => Self::EMBI,
            b"EXTN" => Self::EXTN,
            b"FEAT" => Self::FEAT,
            b"FEDS" => Self::FEDS,
            b"FONT" => Self::FONT,
            b"FUNC" => Self::FUNC,
            b"GEN8" => Self::GEN8,
            b"GLOB" => Self::GLOB,
            b"GMEN" => Self::GMEN,
            b"LANG" => Self::LANG,
            b"OBJT" => Self::OBJT,
            b"OPTN" => Self::OPTN,
            b"PATH" => Self::PATH,
            b"PSEM" => Self::PSEM,
            b"PSYS" => Self::PSYS,
            b"ROOM" => Self::ROOM,
            b"SCPT" => Self::SCPT,
            b"SEQN" => Self::SEQN,
            b"SHDR" => Self::SHDR,
            b"SOND" => Self::SOND,
            b"SPRT" => Self::SPRT,
            b"STRG" => Self::STRG,
            b"TAGS" => Self::TAGS,
            b"TGIN" => Self::TGIN,
            b"TMLN" => Self::TMLN,
            b"TPAG" => Self::TPAG,
            b"TXTR" => Self::TXTR,
            b"UILR" => Self::UILR,
            b"VARI" => Self::VARI,
            _ => {
                let hex = hexdump(&bytes);
                if let Ok(string) = str::from_utf8(&bytes) {
                    bail!("Invalid chunk name {string:?} [{hex}]");
                }
                bail!("Invalid chunk name [{hex}]");
            }
        })
    }

    #[must_use]
    pub const fn as_bytes(self) -> [u8; 4] {
        *match self {
            Self::ACRV => b"ACRV",
            Self::AGRP => b"AGRP",
            Self::AUDO => b"AUDO",
            Self::BGND => b"BGND",
            Self::CODE => b"CODE",
            Self::DAFL => b"DAFL",
            Self::EMBI => b"EMBI",
            Self::EXTN => b"EXTN",
            Self::FEAT => b"FEAT",
            Self::FEDS => b"FEDS",
            Self::FONT => b"FONT",
            Self::FUNC => b"FUNC",
            Self::GEN8 => b"GEN8",
            Self::GLOB => b"GLOB",
            Self::GMEN => b"GMEN",
            Self::LANG => b"LANG",
            Self::OBJT => b"OBJT",
            Self::OPTN => b"OPTN",
            Self::PATH => b"PATH",
            Self::PSEM => b"PSEM",
            Self::PSYS => b"PSYS",
            Self::ROOM => b"ROOM",
            Self::SCPT => b"SCPT",
            Self::SEQN => b"SEQN",
            Self::SHDR => b"SHDR",
            Self::SOND => b"SOND",
            Self::SPRT => b"SPRT",
            Self::STRG => b"STRG",
            Self::TAGS => b"TAGS",
            Self::TGIN => b"TGIN",
            Self::TMLN => b"TMLN",
            Self::TPAG => b"TPAG",
            Self::TXTR => b"TXTR",
            Self::UILR => b"UILR",
            Self::VARI => b"VARI",
        }
    }
}

impl Display for ChunkName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// All chunk elements should implement this trait.
#[expect(private_bounds)]
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
    /// This can be used to distinguish between "present but empty" and "not present at all".
    fn exists(&self) -> bool;
}

/// All chunk elements that represent a collection of elements should implement
/// this trait. The only exceptions are `GEN8` and `OPTN`.
pub trait GMListChunk: GMChunk {
    #[expect(private_bounds)]
    type Element: GMElement;

    #[must_use]
    fn elements(&self) -> impl Iterator<Item = &Self::Element> {
        self.element_refs().map(|(_ref, elem)| elem)
    }

    #[must_use]
    fn element_refs(&self) -> impl Iterator<Item = (GMRef<Self::Element>, &Self::Element)>;

    #[must_use]
    fn elements_mut(&mut self) -> impl Iterator<Item = &mut Self::Element> {
        self.element_refs_mut().map(|(_ref, elem)| elem)
    }

    #[must_use]
    fn element_refs_mut(
        &mut self,
    ) -> impl Iterator<Item = (GMRef<Self::Element>, &mut Self::Element)>;

    fn by_ref(&self, gm_ref: GMRef<Self::Element>) -> Result<&Self::Element>;

    fn by_ref_mut(&mut self, gm_ref: GMRef<Self::Element>) -> Result<&mut Self::Element>;

    fn push(&mut self, element: Self::Element);

    #[must_use]
    fn len(&self) -> usize;

    #[must_use]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait GMNullableListChunk: GMListChunk {
    #[must_use]
    fn all_elements(&self) -> &Vec<Option<Self::Element>>;

    #[must_use]
    fn all_elements_mut(&mut self) -> &mut Vec<Option<Self::Element>>;
}

pub trait GMDirectListChunk: GMListChunk {
    #[must_use]
    fn all_elements(&self) -> &Vec<Self::Element>;

    #[must_use]
    fn all_elements_mut(&mut self) -> &mut Vec<Self::Element>;
}

/// All chunk elements that represent a collection of elements **with a unique name**.
pub trait GMNamedListChunk: GMListChunk<Element: GMNamedElement> {
    fn ref_by_name(&self, name: &str, gm_strings: &Strings) -> Result<GMRef<Self::Element>> {
        for (gm_ref, elem) in self.element_refs() {
            let elem_name: &String = elem.name(gm_strings)?;
            if name == elem_name {
                return Ok(gm_ref);
            }
        }
        Err(err!(
            "Could not find {} with name {name:?}",
            crate::util::fmt::typename::<Self::Element>(),
        ))
    }

    fn by_name(&self, name: &str, gm_strings: &Strings) -> Result<&Self::Element> {
        self.ref_by_name(name, gm_strings)
            .and_then(|elem| self.by_ref(elem))
    }

    fn by_name_mut(&mut self, name: &str, gm_strings: &Strings) -> Result<&mut Self::Element> {
        self.ref_by_name(name, gm_strings)
            .and_then(|elem| self.by_ref_mut(elem))
    }
}

macro_rules! gm_chunk {
    ($name:ident, $chunk_struct:ident) => {
        impl crate::wad::chunk::GMChunk for $chunk_struct {
            const NAME: crate::wad::chunk::ChunkName = crate::wad::chunk::ChunkName::$name;

            fn exists(&self) -> bool {
                self.exists
            }
        }
    };
}

macro_rules! gm_list_chunk {
    ($name:ident, $chunk_struct:ident, $elem_type:ty,nullable) => {
        crate::wad::chunk::gm_chunk!($name, $chunk_struct);

        impl crate::wad::chunk::GMListChunk for $chunk_struct {
            type Element = $elem_type;

            fn element_refs(&self) -> impl Iterator<Item = (GMRef<Self::Element>, &Self::Element)> {
                self.elems
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, elem)| Some((idx.into(), elem.as_ref()?)))
            }

            fn element_refs_mut(
                &mut self,
            ) -> impl Iterator<Item = (GMRef<Self::Element>, &mut Self::Element)> {
                self.elems
                    .iter_mut()
                    .enumerate()
                    .filter_map(|(idx, elem)| Some((idx.into(), elem.as_mut()?)))
            }

            fn by_ref(&self, gm_ref: GMRef<Self::Element>) -> Result<&Self::Element> {
                gm_ref.opt_resolve(&self.elems)
            }

            fn by_ref_mut(&mut self, gm_ref: GMRef<Self::Element>) -> Result<&mut Self::Element> {
                gm_ref.opt_resolve_mut(&mut self.elems)
            }

            fn push(&mut self, element: Self::Element) {
                self.elems.push(Some(element));
            }

            fn len(&self) -> usize {
                self.elems.len()
            }
        }

        impl crate::wad::chunk::GMNullableListChunk for $chunk_struct {
            fn all_elements(&self) -> &Vec<Option<Self::Element>> {
                &self.elems
            }

            fn all_elements_mut(&mut self) -> &mut Vec<Option<Self::Element>> {
                &mut self.elems
            }
        }
    };

    ($name:ident, $chunk_struct:ident, $elem_type:ty,direct) => {
        crate::wad::chunk::gm_chunk!($name, $chunk_struct);

        impl crate::wad::chunk::GMListChunk for $chunk_struct {
            type Element = $elem_type;

            fn element_refs(&self) -> impl Iterator<Item = (GMRef<Self::Element>, &Self::Element)> {
                self.elems
                    .iter()
                    .enumerate()
                    .map(|(idx, elem)| (idx.into(), elem))
            }

            fn element_refs_mut(
                &mut self,
            ) -> impl Iterator<Item = (GMRef<Self::Element>, &mut Self::Element)> {
                self.elems
                    .iter_mut()
                    .enumerate()
                    .map(|(idx, elem)| (idx.into(), elem))
            }

            fn by_ref(&self, gm_ref: GMRef<Self::Element>) -> Result<&Self::Element> {
                gm_ref.resolve(&self.elems)
            }

            fn by_ref_mut(&mut self, gm_ref: GMRef<Self::Element>) -> Result<&mut Self::Element> {
                gm_ref.resolve_mut(&mut self.elems)
            }

            fn push(&mut self, element: Self::Element) {
                self.elems.push(element);
            }

            fn len(&self) -> usize {
                self.elems.len()
            }
        }

        impl crate::wad::chunk::GMDirectListChunk for $chunk_struct {
            fn all_elements(&self) -> &Vec<Self::Element> {
                &self.elems
            }

            fn all_elements_mut(&mut self) -> &mut Vec<Self::Element> {
                &mut self.elems
            }
        }
    };
}

macro_rules! gm_named_list_chunk {
    ($name:ident, $chunk_struct:ident, $elem_type:ty, $tspmo:ident) => {
        crate::wad::chunk::gm_list_chunk!($name, $chunk_struct, $elem_type, $tspmo);

        impl crate::wad::chunk::GMNamedListChunk for $chunk_struct {}

        impl crate::wad::elem::GMNamedElement for $elem_type {
            fn name_ref(&self) -> crate::wad::GMRef<String> {
                self.name
            }
        }
    };
}

pub(crate) use gm_chunk;
pub(crate) use gm_list_chunk;
pub(crate) use gm_named_list_chunk;
