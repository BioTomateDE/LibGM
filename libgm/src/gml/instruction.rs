mod asset_reference;
mod category;
mod code_variable;
mod comparison_type;
mod data_type;
mod instance_type;
mod push_value;
mod variable_type;

use crate::gamemaker::{elements::function::GMFunction, reference::GMRef};

pub use asset_reference::AssetReference;
pub use category::Category;
pub use code_variable::CodeVariable;
pub use comparison_type::ComparisonType;
pub use data_type::DataType;
pub use instance_type::InstanceType;
pub use push_value::PushValue;
pub use variable_type::VariableType;

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    /// Converts the top of the stack from one type to another.
    Convert { from: DataType, to: DataType },

    /// Pops two values from the stack, multiplies them, and pushes the result.
    Multiply {
        multiplicand: DataType,
        multiplier: DataType,
    },

    /// Pops two values from the stack, divides them, and pushes the result.
    /// The second popped value is divided by the first popped value.
    Divide {
        dividend: DataType,
        divisor: DataType,
    },

    /// Pops two values from the stack, performs a GML `div` operation (division with remainder), and pushes the result.
    /// The second popped value is divided (with remainder) by the first popped value.
    Remainder {
        dividend: DataType,
        divisor: DataType,
    },

    /// Pops two values from the stack, performs a GML `mod` operation (`%`), and pushes the result.
    /// The second popped value is modulo'd against the first popped value.
    Modulus {
        dividend: DataType,
        divisor: DataType,
    },

    /// Pops two values from the stack, adds them, and pushes the result.
    Add { augend: DataType, addend: DataType },

    /// Pops two values from the stack, **subtracts** them, and pushes the result.
    /// The second popped value is subtracted by the first popped value.
    Subtract {
        minuend: DataType,
        subtrahend: DataType,
    },

    /// Pops two values from the stack, performs an **AND** operation, and pushes the result.
    /// This can be done bitwise or logically.
    And { lhs: DataType, rhs: DataType },

    /// Pops two values from the stack, performs an **OR** operation, and pushes the result.
    /// This can be done bitwise or logically.
    Or { lhs: DataType, rhs: DataType },

    /// Pops two values from the stack, performs an **XOR** operation, and pushes the result.
    /// This can be done bitwise or logically.
    Xor { lhs: DataType, rhs: DataType },

    /// Negates the top value of the stack (as in, multiplies it with negative one).
    Negate { data_type: DataType },

    /// Performs a boolean or bitwise NOT operation on the top value of the stack (modifying it).
    Not { data_type: DataType },

    /// Pops two values from the stack, performs a bitwise left shift operation (`<<`), and pushes the result.
    /// The second popped value is shifted left by the first popped value.
    ShiftLeft {
        value: DataType,
        shift_amount: DataType,
    },

    /// Pops two values from the stack, performs a bitwise right shift operation (`>>`), and pushes the result.
    /// The second popped value is shifted right by the first popped value.
    ShiftRight {
        value: DataType,
        shift_amount: DataType,
    },

    /// Pops two values from the stack, compares them using a [`ComparisonType`], and pushes a boolean result.
    Compare {
        lhs: DataType,
        rhs: DataType,
        comparison_type: ComparisonType,
    },

    /// Pops a value from the stack, and generally stores it in a variable, array, or otherwise.
    /// Has an alternate mode that can swap values around on the stack.
    /// TODO(weak): type1 and type2 are bad names and are probably redundant values
    Pop {
        variable: CodeVariable,
        type1: DataType,
        type2: DataType,
    },

    /// Swaps values around on the stack
    PopSwap { is_array: bool },

    /// Duplicates values on the stack.
    ///
    /// The specified `size` is the total size of all
    /// elements that should be duplicated.
    Duplicate { data_type: DataType, size: u8 },

    /// Swaps values around on the stack.
    ///
    /// First, elements with a total size of `size1` are popped into a temporary "top stack".
    /// Then, elements with a total size of `size2` are popped into a temporary "bottom stack".
    /// Afterward, the "bottom stack" is pushed.
    /// And lastly, the "top stack" is pushed.
    DuplicateSwap {
        data_type: DataType,
        size1: u8,
        size2: u8,
    },

    /// Pops a value from the stack, and returns from the current
    /// function/script with that value as the return value.
    Return,

    /// Returns from the current function/script/event with no return value.
    Exit,

    /// Pops a value from the stack, and discards it.
    PopDiscard { data_type: DataType },

    /// Branches (jumps) to another instruction in the code entry.
    Branch { jump_offset: i32 },

    /// Pops a boolean/int32 value from the stack. If true/nonzero, branches (jumps) to another instruction in the code entry.
    BranchIf { jump_offset: i32 },

    /// Pops a boolean/int32 value from the stack. If false/zero, branches (jumps) to another instruction in the code entry.
    BranchUnless { jump_offset: i32 },

    /// Pushes a `with` context, used for GML `with` statements, to the VM environment/self instance stack.
    PushWithContext { jump_offset: i32 },

    /// Pops/ends a `with` context, used for GML `with` statements, from the VM environment/self instance stack.
    /// This instruction will branch to its encoded address until no longer iterating instances, where the context will finally be gone for good.
    /// If a flag is encoded in this instruction, then this will always terminate the loops, and branch to the encoded address.
    PopWithContext { jump_offset: i32 },

    /// `PopWithContext` but with `PopEnvExitMagic`
    PopWithContextExit,

    /// Pushes a constant value onto the stack. Can vary in size depending on value type.
    Push { value: PushValue },

    /// Pushes a value stored in a local variable onto the stack.
    PushLocal { variable: CodeVariable },

    /// Pushes a value stored in a global variable onto the stack.
    PushGlobal { variable: CodeVariable },

    /// Pushes a value stored in a GameMaker builtin variable onto the stack.
    PushBuiltin { variable: CodeVariable },

    /// Pushes an immediate signed 32-bit integer value onto the stack, encoded as a signed 16-bit integer.
    PushImmediate { integer: i16 },

    /// Calls a GML script/function, using its ID. Arguments are prepared prior to this instruction, in reverse order.
    /// Argument count is encoded in this instruction. Arguments are popped off of the stack.
    Call {
        function: GMRef<GMFunction>,
        argument_count: u16,
    },

    /// Pops two values off of the stack, and then calls a GML script/function using those values, representing
    /// the "self" instance to be used when calling, as well as the reference to the function being called.
    /// Arguments are dealt with identically to "call".
    CallVariable { argument_count: u16 },

    /// Verifies an array index is within proper bounds, typically for multidimensional arrays.
    CheckArrayIndex,

    /// Pops two values from the stack, those being an index and an array reference.
    /// Then, pushes the value stored at the passed-in array at the desired index.
    /// That is, this is used only with multidimensional arrays, for the final/last index operation.
    PushArrayFinal,

    /// Pops three values from the stack, those being an index, an array reference, and a value.
    /// Then, assigns the value to the array at the specified index.
    PopArrayFinal,

    /// Pops two values from the stack, those being an array reference and an index.
    /// Then, pushes a new array reference from the passed-in array at the desired index,
    /// with the expectation that it will be further indexed into.
    /// That is, this is used only with multidimensional arrays,
    /// for all index operations from the second through the second to last.
    PushArrayContainer,

    /// Sets a global variable in the VM (popped from stack), designated for
    /// tracking the now-deprecated array copy-on-write functionality in GML.
    /// The value used is specific to certain locations in scripts.
    /// When array copy-on-write functionality is disabled, this extended opcode is not used.
    SetArrayOwner,

    /// Pushes a boolean value to the stack, indicating whether static initialization
    /// has already occurred for this function (true), or otherwise false.
    HasStaticInitialized,

    /// Marks the current function to no longer be able to enter its own static initialization.
    /// This can either occur at the beginning or end of a static block,
    /// depending on whether "`AllowReentrantStatic`" is enabled by a game's developer
    /// (enabled by default before GameMaker 2024.11; disabled by default otherwise).
    SetStaticInitialized,

    /// Keeps track of an array reference temporarily. Used in multidimensional array compound assignment statements.
    /// Presumed to be used for garbage collection purposes.
    SaveArrayReference,

    /// Restores a previously-tracked array reference.
    /// Used in multidimensional array compound assignment statements.
    /// Presumed to be used for garbage collection purposes.
    RestoreArrayReference,

    /// Pops a value from the stack, and pushes a boolean result.
    /// The result is true if a "nullish" value, such as undefined or GML's `pointer_null`.
    IsNullishValue,

    /// Pushes an asset reference to the stack, encoded in an integer. Includes asset type and index.
    PushReference { asset_reference: AssetReference },
}

impl Instruction {
    /// Gets the instruction size in bytes.
    /// This size includes extra data like integers, floats, variable references, etc.
    #[must_use]
    pub const fn size(&self) -> u8 {
        match self {
            Self::Push { value } => match value {
                PushValue::Int16(_) => 4,
                PushValue::Int64(_) | PushValue::Double(_) => 12,
                _ => 8,
            },
            Self::Pop { .. }
            | Self::PushLocal { .. }
            | Self::PushGlobal { .. }
            | Self::PushBuiltin { .. }
            | Self::Call { .. }
            | Self::PushReference { .. } => 8,
            _ => 4,
        }
    }

    /// Attempts to extract a [`CodeVariable`] from  the instruction.
    /// This can succeed for `Push` and will always succeed for
    /// `PushGlobal`, `PushLocal`, `PushBuiltin` and `Pop`.
    #[must_use]
    pub const fn variable(&self) -> Option<&CodeVariable> {
        match self {
            Self::Pop { variable, .. }
            | Self::Push { value: PushValue::Variable(variable) }
            | Self::PushLocal { variable }
            | Self::PushGlobal { variable }
            | Self::PushBuiltin { variable } => Some(variable),
            _ => None,
        }
    }

    /// Attempts to extract a `GMRef<GMFunction>` from the instruction.
    /// This can succeed for `Push` and `PushReference` and will always succeed for `Call`.
    #[must_use]
    pub const fn function(&self) -> Option<GMRef<GMFunction>> {
        match self {
            Self::Push { value: PushValue::Function(function) }
            | Self::Call { function, .. }
            | Self::PushReference {
                asset_reference: AssetReference::Function(function),
            } => Some(*function),
            _ => None,
        }
    }

    /// Attempts to extract a jump offset in bytes from the instruction.
    /// This will always succeed for `Branch`, `BranchIf`, `BranchUnless`, `PushWithContext` and
    /// `PopWithContext`.
    #[must_use]
    pub const fn jump_offset(&self) -> Option<i32> {
        match self {
            Self::Branch { jump_offset }
            | Self::BranchIf { jump_offset }
            | Self::BranchUnless { jump_offset }
            | Self::PushWithContext { jump_offset }
            | Self::PopWithContext { jump_offset } => Some(*jump_offset),
            _ => None,
        }
    }

    /// Attempts to extract the first (or the only) data type from the instruction.
    ///
    /// For binary operations, this will be RHS.
    ///
    /// NOTE: Right now, it's kind of arbitary which instructions' data types are
    /// deemed "relevant enough" to return them and which don't really belong
    /// (does return: `PushLocal` [Variable], does not return: `PopSwap` [Int16]).
    /// This will be changed in the future **if I get feedback pls**.
    #[must_use]
    pub const fn type1(&self) -> Option<DataType> {
        Some(match self {
            Self::Convert { from, .. } => *from,
            Self::Multiply { multiplier, .. } => *multiplier,
            Self::Divide { divisor, .. }
            | Self::Remainder { divisor, .. }
            | Self::Modulus { divisor, .. } => *divisor,
            Self::Add { addend, .. } => *addend,
            Self::Subtract { subtrahend, .. } => *subtrahend,
            Self::And { rhs, .. }
            | Self::Or { rhs, .. }
            | Self::Xor { rhs, .. }
            | Self::Compare { rhs, .. } => *rhs,
            Self::Negate { data_type }
            | Self::Not { data_type }
            | Self::Duplicate { data_type, .. }
            | Self::DuplicateSwap { data_type, .. }
            | Self::PopDiscard { data_type } => *data_type,
            Self::ShiftLeft { shift_amount, .. } | Self::ShiftRight { shift_amount, .. } => {
                *shift_amount
            },
            Self::Pop { type1, .. } => *type1,
            Self::Push { value } => value.data_type(),
            Self::PushLocal { .. } | Self::PushGlobal { .. } | Self::PushBuiltin { .. } => {
                DataType::Variable
            },
            Self::PushImmediate { .. } => DataType::Int16,
            Self::Call { .. } => DataType::Int32,
            Self::CallVariable { .. } => DataType::Variable,
            _ => return None,
        })
    }

    /// Attempts to return the second data type of this instruction.
    ///
    /// For binary operations, this will be LHS.
    #[must_use]
    pub const fn type2(&self) -> Option<DataType> {
        Some(match self {
            Self::Convert { to, .. } => *to,
            Self::Multiply { multiplicand, .. } => *multiplicand,
            Self::Divide { dividend, .. }
            | Self::Remainder { dividend, .. }
            | Self::Modulus { dividend, .. } => *dividend,
            Self::Add { augend, .. } => *augend,
            Self::Subtract { minuend, .. } => *minuend,
            Self::And { lhs, .. }
            | Self::Or { lhs, .. }
            | Self::Xor { lhs, .. }
            | Self::Compare { lhs, .. } => *lhs,
            Self::ShiftLeft { value, .. } | Self::ShiftRight { value, .. } => *value,
            Self::Pop { type2, .. } => *type2,
            _ => return None,
        })
    }
}
