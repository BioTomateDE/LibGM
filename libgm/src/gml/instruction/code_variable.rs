use crate::{
    gml::instruction::{InstanceType, VariableType},
    prelude::GMRef,
    wad::elements::variable::GMVariable,
};

/// A variable reference in an instruction.
/// Contains the actual variable ref as well as instance type and variable type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeVariable {
    pub variable: GMRef<GMVariable>,
    pub variable_type: VariableType,
    pub instance_type: InstanceType,

    /// TODO: when does this happen?
    pub is_int32: bool,
}
