// SPDX-License-Identifier: GPL-3.0-only
use crate::gml::instruction::InstanceType;
use crate::gml::instruction::VariableType;
use crate::prelude::GMRef;
use crate::wad::elem::variable::Variable;

/// A variable reference in an instruction.
/// Contains the actual variable ref as well as instance type and variable type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodeVariable {
    pub variable: GMRef<Variable>,
    pub variable_type: VariableType,
    pub instance_type: InstanceType,

    /// TODO: when does this happen?
    pub is_int32: bool,
}

impl CodeVariable {
    /// Creates a new [`CodeVariable`] with [`VariableType::Normal`] and `is_int32 = false`.
    #[must_use]
    pub fn new(variable: GMRef<Variable>, instance_type: InstanceType) -> Self {
        Self {
            variable,
            variable_type: VariableType::Normal,
            instance_type,
            is_int32: false,
        }
    }
}
