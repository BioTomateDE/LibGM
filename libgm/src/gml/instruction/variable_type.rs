use macros::num_enum;

/// How a variable is supposed to be used in an instruction.
#[num_enum(u8)]
pub enum VariableType {
    /// Used for normal single-dimension array variables.
    Array = 0x00,

    /// Used when referencing a variable on another variable, e.g. a chain reference.
    StackTop = 0x80,

    /// Used for normal variables, without any arrays or chain references.
    Normal = 0xA0,

    /// Used when referencing variables on room instance IDs, e.g. something like `inst_01ABCDEF.x` in GML.        
    Instance = 0xE0,

    /// (GMS2.3+) Used in tandem with multi-dimensional array push operations (`PushArrayFinal`).
    MultiPush = 0x10,

    /// (GMS2.3+) Used in tandem with multi-dimensional array push and pop operations (`PushArrayFinal`, `PopArrayFinal`).
    MultiPop = 0x90,
}
