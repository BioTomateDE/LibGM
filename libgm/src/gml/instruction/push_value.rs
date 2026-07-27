// SPDX-License-Identifier: GPL-3.0-only
use crate::gml::Instruction;
use crate::gml::instruction::CodeVariable;
use crate::gml::instruction::DataType;
use crate::prelude::GMRef;
use crate::wad::elem::function::Function;

/// A value to push to the stack.
/// Used in [`Push`] instructions.
///
/// [`Push`]: Instruction::Push
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PushValue {
    Short(i16),
    Int(i32),
    Long(i64),
    Double(f64),
    Bool(bool),
    String(GMRef<String>),
    Variable(CodeVariable),
    Function(GMRef<Function>),
}

impl PushValue {
    /// This value's [`DataType`].
    ///
    /// Note that functions are represented as `Int32`.
    /// Some variables are also represented as `Int32` (idk when tho).
    #[must_use]
    pub const fn data_type(&self) -> DataType {
        match self {
            Self::Short(_) => DataType::Short,
            Self::Int(_) | Self::Function(_) => DataType::Int,
            Self::Variable(var) if var.is_int32 => DataType::Int,
            Self::Long(_) => DataType::Long,
            Self::Double(_) => DataType::Double,
            Self::Bool(_) => DataType::Bool,
            Self::String(_) => DataType::String,
            Self::Variable(_) => DataType::Variable,
        }
    }

    /// An approximate boolean representation for this [`PushValue`].
    ///
    /// Warning: This function does not fully conform with the GameMaker
    /// standards (yet).
    #[must_use]
    pub(crate) const fn as_bool(&self) -> Option<bool> {
        match self {
            &Self::Short(int) => Some(int > 0),
            &Self::Int(int) => Some(int > 0),
            &Self::Long(int) => Some(int > 0),
            &Self::Double(float) => Some(float > 0.5),
            &Self::Bool(bool) => Some(bool),
            Self::String(_) | Self::Function(_) | Self::Variable(_) => None,
        }
    }
}

impl Instruction {
    /// Attempts to extract a [`PushValue`] from this instruction.
    #[must_use]
    pub const fn push_value(&self) -> Option<PushValue> {
        Some(match *self {
            Self::Push { value } => value,
            Self::PushLocal { variable }
            | Self::PushGlobal { variable }
            | Self::PushBuiltin { variable } => PushValue::Variable(variable),
            Self::PushImmediate { integer } => PushValue::Short(integer),
            _ => return None,
        })
    }
}
