// SPDX-License-Identifier: GPL-3.0-only
use std::fmt;

use crate::gml::instruction::InstanceType;
use crate::prelude::*;

/// The kind a [`GMVariable`] can be.
///
/// This is kinda the scope of the variable.
///
/// Only used in WAD 15+.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum VariableType {
    /// This can represent one of the following:
    /// * [`InstanceType::Self_`]
    /// * [`InstanceType::Other`]
    /// * [`InstanceType::Builtin`]
    /// * [`InstanceType::StackTop`]
    /// * [`InstanceType::GameObject`]
    /// * [`InstanceType::Instance`]
    ///
    /// In fact, this can represent any instance type if the variable reference type is not [normal].
    ///
    /// [normal]: crate::gml::instruction::ReferenceType::Normal
    Self_,

    /// Global scope ([`InstanceType::Global`]).
    /// This is the only instance/variable type that makes complete sense.
    Global,

    /// Local scope ([`InstanceType::Local`]).
    /// Only presists until the end of the code entry execution.
    Local,

    /// Used for argument variables (`argument0` - `argument15`), which corresponds to [`InstanceType::Argument`].
    /// This is *not* used for other builtin variables like `x`, `room_speed`, etc.
    Builtin,

    /// DOCME: I don't care about statics.
    Static,
}

impl VariableType {
    /// Decodes a variable type from the given raw value.
    ///
    /// This is the same logic as decoding instance types, just limited/converted for variable types.
    pub fn decode(raw: i32) -> Result<Self> {
        Ok(match raw {
            -1 => Self::Self_,
            -5 => Self::Global,
            -6 => Self::Builtin,
            -7 => Self::Local,
            -16 => Self::Static,
            _ => bail!("Invalid variable type {raw} (0x{raw:04X})"),
        })
    }

    /// Encodes this variable type into an [`i32`].
    ///
    /// This is the same logic as encoding instance types, just limited/converted for variable types.
    #[must_use]
    pub const fn encode(self) -> i32 {
        match self {
            Self::Self_ => -1,
            Self::Global => -5,
            Self::Builtin => -6,
            Self::Local => -7,
            Self::Static => -16,
        }
    }

    /// Primitively converts this variable type as an instance type.
    ///
    /// Variable type `builtin` is converted to instance type `builtin` (instead of `argument`).
    ///
    /// It's impossible to determine the *actual* instance type,
    /// since this variable type stores less information (lots of stuff is just converted to `self` for some reason).
    #[must_use]
    pub const fn as_instance_type(self) -> InstanceType {
        match self {
            Self::Self_ => InstanceType::Self_,
            Self::Global => InstanceType::Global,
            Self::Local => InstanceType::Local,
            Self::Builtin => InstanceType::Builtin,
            Self::Static => InstanceType::Static,
        }
    }
}

impl fmt::Debug for VariableType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::Self_ => "Self",
            Self::Global => "Global",
            Self::Local => "Local",
            Self::Builtin => "Builtin",
            Self::Static => "Static",
        };
        f.write_str(s)
    }
}
