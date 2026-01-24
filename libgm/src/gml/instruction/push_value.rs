use crate::{
    gamemaker::elements::function::GMFunction,
    gml::instruction::{CodeVariable, DataType},
    prelude::GMRef,
};

/// A value to push to the stack. Used in `Push` instructions.
#[derive(Debug, Clone, PartialEq)]
pub enum PushValue {
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Double(f64),
    Boolean(bool),
    String(String),
    Function(GMRef<GMFunction>),
    Variable(CodeVariable),
}

impl PushValue {
    #[must_use]
    pub const fn data_type(&self) -> DataType {
        match self {
            Self::Int16(_) => DataType::Int16,
            Self::Int32(_) | Self::Function(_) => DataType::Int32, // Functions are not a "real" gm type; they're always int32
            Self::Variable(var) if var.is_int32 => DataType::Int32, // no idea when this happens
            Self::Int64(_) => DataType::Int64,
            Self::Double(_) => DataType::Double,
            Self::Boolean(_) => DataType::Boolean,
            Self::String(_) => DataType::String,
            Self::Variable(_) => DataType::Variable,
        }
    }

    /// Warning: This function does not fully conform with the GameMaker standards (yet).
    #[must_use]
    pub const fn as_bool(&self) -> Option<bool> {
        match self {
            &Self::Int16(int) => Some(int > 0),
            &Self::Int32(int) => Some(int > 0),
            &Self::Int64(int) => Some(int > 0),
            &Self::Double(float) => Some(float > 0.5),
            &Self::Boolean(bool) => Some(bool),
            Self::String(_) | Self::Function(_) | Self::Variable(_) => None,
        }
    }
}
