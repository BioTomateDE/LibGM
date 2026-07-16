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

/// The number of all known GameMaker chunks (excluding debug chunks).
pub const KNOWN_CHUNK_COUNT: usize = 35;

/// The order of chunks in a data file.
/// Also determines which chunks exist.
#[derive(Debug, Clone)]
pub struct ChunkOrder(pub(crate) Vec<ChunkName>);

impl ChunkOrder {
    /// Creates a new empty chunk order.
    ///
    /// This will throw an error when serializing, so make sure to fill it with chunks.
    #[must_use]
    pub const fn new_empty() -> Self {
        Self(Vec::new())
    }

    /// Tries to create a chunk order from a vector of chunk names.
    ///
    /// This can fail if there are duplicate chunks.
    pub fn from_vec(vector: Vec<ChunkName>) -> Result<Self> {
        let count = vector.len();
        let known = KNOWN_CHUNK_COUNT;
        if count > known {
            bail!("Vector has {count} elements which is larger than the known chunk count {known}");
        }

        // check for duplicates
        let mut seen: u64 = 0;
        for &chunk in &vector {
            // this works because there's less than 65 ChunkName variants
            let bit = 1u64 << (chunk as u32);
            if seen & bit != 0 {
                bail!("Duplicate chunk {chunk} in vector");
            }
            seen |= bit;
        }

        // ok you're fine to go
        Ok(Self(vector))
    }

    /// The count/length of chunks in this chunk order.
    ///
    /// This can never be greater than the amount of chunks
    /// (variant count of [`Chunk`]).
    ///
    /// Usually, it will also be non-zero, as data files with no chunks are invalid.
    /// However, it is possible to artificially create an empty
    /// chunk order using [`Self::new_empty`] or [`Self::from_vec`].
    #[doc(alias = "len")]
    #[must_use]
    pub const fn count(&self) -> usize {
        self.0.len()
    }

    /// Whether this chunk order contains no chunks.
    ///
    /// This will usually be false, as data files with no chunks are invalid.
    /// However, it is possible to artificially create an empty
    /// chunk order using [`Self::new_empty`] or [`Self::from_vec`].
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Whether the given chunk exists in this chunk order.
    #[doc(alias = "contains")]
    #[must_use]
    pub fn has(&self, chunk: ChunkName) -> bool {
        self.0.contains(&chunk)
    }

    /// The index of given chunk in this chunk order, if it exists.
    #[must_use]
    pub fn find(&self, chunk: ChunkName) -> Option<usize> {
        self.0.iter().position(|&c| c == chunk)
    }

    /// The chunk at the specified index of this chunk order, if it exists.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<ChunkName> {
        self.0.get(index).copied()
    }

    /// Adds a chunk to the end of the chunk order.
    ///
    /// Returns whether the operation was successful:
    /// If the chunk already exists, it will not be inserted and `false` will be returned.
    #[doc(alias = "add")]
    pub fn push(&mut self, chunk: ChunkName) -> bool {
        if self.has(chunk) {
            false
        } else {
            self.0.push(chunk);
            true
        }
    }

    /// Removes a chunk from the chunk order.
    ///
    /// Returns whether the operation was successful:
    /// If the chunk did not exist in the first place, `false` will be returned.
    pub fn remove(&mut self, chunk: ChunkName) -> bool {
        if let Some(idx) = self.find(chunk) {
            self.0.remove(idx);
            true
        } else {
            false
        }
    }

    /// Inserts a chunk into the chunk order at the specified index.
    ///
    /// Returns whether the operation was successful:
    /// If the chunk already exists, it will not be inserted and `false` will be returned.
    ///
    /// Note that the index logic is saturating:
    /// If you provide an index out of bounds, it will clamp to the end of the chunk order.
    #[must_use]
    pub fn insert(&mut self, index: usize, chunk: ChunkName) -> bool {
        if self.has(chunk) {
            false
        } else {
            let idx: usize = self.count().min(index);
            self.0.insert(idx, chunk);
            true
        }
    }

    /// Moves a chunk from its current position in the chunk order to the specified index.
    ///
    /// Returns whether the operation was successful:
    /// If the chunk exists and could therefore be moved around, `true` will be returned.
    ///
    /// Note that the index logic is saturating:
    /// If you provide an index out of bounds, it will clamp to the end of the chunk order.
    pub fn move_to(&mut self, chunk: ChunkName, index: usize) -> bool {
        if self.remove(chunk) {
            let idx: usize = self.count().min(index);
            self.0.insert(idx, chunk);
            true
        } else {
            false
        }
    }

    pub fn iter(&'_ self) -> std::iter::Copied<std::slice::Iter<'_, ChunkName>> {
        self.0.iter().copied()
    }
}

impl IntoIterator for ChunkOrder {
    type IntoIter = std::vec::IntoIter<ChunkName>;
    type Item = ChunkName;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a ChunkOrder {
    type IntoIter = std::iter::Copied<std::slice::Iter<'a, ChunkName>>;
    type Item = ChunkName;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().copied()
    }
}

/// All chunk elements should implement this trait.
#[expect(private_bounds)]
pub trait GMChunk: GMElement + Default {
    /// The four character GameMaker chunk name (GEN8, STRG, VARI, etc.).
    const NAME: ChunkName;
}

/// All chunk elements that represent a collection of elements should implement
/// this trait. The only exceptions are `GEN8` and `OPTN`.
pub trait GMListChunk: GMChunk {
    // usually also GMElement
    type Element;

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

    fn push(&mut self, element: Self::Element) -> GMRef<Self::Element>;

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

macro_rules! gm_list_chunk {
    ($name:ident, $chunk_struct:ident, $elem_type:ty,nullable) => {
        impl crate::wad::chunk::GMChunk for $chunk_struct {
            const NAME: crate::wad::chunk::ChunkName = crate::wad::chunk::ChunkName::$name;
        }

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

            fn push(&mut self, element: Self::Element) -> GMRef<Self::Element> {
                self.elems.push(Some(element));
                (self.elems.len() - 1).into()
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
        impl crate::wad::chunk::GMChunk for $chunk_struct {
            const NAME: crate::wad::chunk::ChunkName = crate::wad::chunk::ChunkName::$name;
        }

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

            fn push(&mut self, element: Self::Element) -> GMRef<Self::Element> {
                self.elems.push(element);
                (self.elems.len() - 1).into()
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

pub(crate) use gm_list_chunk;
pub(crate) use gm_named_list_chunk;
