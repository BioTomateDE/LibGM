// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;

/// How a variable is supposed to be used in an instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VariableType {
    /// Used for normal variables, without any arrays or chain references.
    Normal,

    /// Used for normal single-dimension array variables.
    ///
    /// When accessing this variable (usually push or pop),
    /// the engine will first pop the index, then the instance type and then perform the operation.
    ///
    /// An array access `value = global.array[index]` will be compiled to:
    /// ```text
    /// pushim -5
    /// push.v self.index
    /// push.v [array]global.array
    /// pop.v.v self.value
    /// ```
    /// The value -5 corresponds to the instance type Global.
    Array,

    /// Used when referencing a variable on another variable, e.g. a chain reference.
    ///
    /// When accessing this variable (usually push or pop),
    /// the engine will first pop an instance type and then perform the operation.
    ///
    /// A chain reference `a = b.c` will be compiled to:
    /// ```text
    /// push.v self.b
    /// conv.v.i
    /// push.v [stacktop]self.c
    /// pop.v.v self.a
    /// ```
    StackTop,

    /// Used when referencing variables on room instance IDs, e.g. something
    /// like `inst_01ABCDEF.x` in GML.
    ///
    /// DOCME: more info
    Instance,

    /// (GMS2.3+) Used in tandem with multidimensional array push operations (`PushArrayFinal`).
    MultiPush,

    /// (GMS2.3+) Used in tandem with multidimensional array push and pop
    /// operations (`PushArrayFinal`, `PopArrayFinal`).
    MultiPop,
}

impl VariableType {
    pub fn from_u8(raw: u8) -> Result<Self> {
        Ok(match raw {
            0xA0 => Self::Normal,
            0x00 => Self::Array,
            0x80 => Self::StackTop,
            0xE0 => Self::Instance,
            0x10 => Self::MultiPush,
            0x90 => Self::MultiPop,
            _ => bail!("Invalid Variable Reference Type {raw} ({raw:04X})"),
        })
    }

    #[must_use]
    pub const fn as_u8(self) -> u8 {
        match self {
            Self::Normal => 0xA0,
            Self::Array => 0x00,
            Self::StackTop => 0x80,
            Self::Instance => 0xE0,
            Self::MultiPush => 0x10,
            Self::MultiPop => 0x90,
        }
    }
}
