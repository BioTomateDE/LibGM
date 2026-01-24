use crate::{
    gamemaker::elements::variable::GMVariable,
    gml::instruction::{InstanceType, VariableType},
    prelude::GMRef,
};

/// A variable reference in an instruction.
/// Contains the actual variable ref as well as instance type and variable type.
#[derive(Debug, Clone, PartialEq)]
pub struct CodeVariable {
    pub variable: GMRef<GMVariable>,
    pub variable_type: VariableType,
    pub instance_type: InstanceType,

    /// TODO: when does this happen?
    pub is_int32: bool,
}
