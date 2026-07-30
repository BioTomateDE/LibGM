// SPDX-License-Identifier: GPL-3.0-only
use crate::gml::instruction::InstanceType;
use crate::gml::instruction::ReferenceType;
use crate::prelude::GMRef;
use crate::wad::elem::variable::Variable;

/// A variable reference in an instruction.
///
/// Contains the reference to the actual variable in `VARI` as well as the instance type and variable reference type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodeVariable {
    pub variable: GMRef<Variable>,
    pub ref_type: ReferenceType,
    pub instance_type: InstanceType,

    /// DOCME: when does this happen?
    /// apparently something with variable hashes in GM 2024.something+.
    /// more research needed.
    pub is_int32: bool,
}

impl CodeVariable {
    /// Creates a new [`CodeVariable`] with [`ReferenceType::Normal`] and `is_int32 = false`.
    #[must_use]
    pub const fn new(variable: GMRef<Variable>, instance_type: InstanceType) -> Self {
        Self {
            variable,
            ref_type: ReferenceType::Normal,
            instance_type,
            is_int32: false,
        }
    }
}
