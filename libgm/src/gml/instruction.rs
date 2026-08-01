// SPDX-License-Identifier: GPL-3.0-only
//! Everything related to GML instructions is in here,
//! including the important [`Instruction`] type.

mod asset_reference;
mod binary;
mod code_variable;
mod comparison_type;
mod data_type;
mod instance_type;
mod push_value;
mod ref_type;
mod variable_type;

pub use self::asset_reference::AssetReference;
pub use self::binary::BinaryOp;
pub use self::code_variable::CodeVariable;
pub use self::comparison_type::ComparisonType;
pub use self::data_type::DataType;
pub use self::instance_type::InstanceType;
pub use self::push_value::PushValue;
pub use self::ref_type::ReferenceType;
pub use self::variable_type::VariableType;
use crate::wad::GMRef;
use crate::wad::elem::function::Function;

/// A GameMaker VM Instruction.
///
/// This is the most important data type for GML.
///
/// For more information on GML, see the [module level documentation].
///
/// [module level documentation]: crate::gml
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    /// Pops two values from the stack, performs a mathematical,
    /// bitwise or logical operation on them, and pushes the result.
    ///
    /// This enum variant covers multiple opcodes for convenience.
    Binary { op: BinaryOp, lhs: DataType, rhs: DataType },

    /// Converts the top of the stack from one type to another.
    ///
    /// DOCME: conversion logic for different permutations
    Convert { from: DataType, to: DataType },

    /// **Negates** the top value of the stack (as in, multiplies it with -1).
    /// Negating a string or boolean results in 0.
    Negate { data_type: DataType },

    /// Pops one value from the stack, performs a **NOT** operation, and pushes the result.
    /// This can be done bitwise or logically, depending on the data type.
    /// Performing NOT on a string results in an engine error.
    Not { data_type: DataType },

    /// Pops two values from the stack, **compares** them using a
    /// [`ComparisonType`], and pushes a boolean result ([`DataType::Bool`]).
    /// The left hand side is popped first, the right hand side is popped second.
    Compare { lhs: DataType, rhs: DataType, comparison_type: ComparisonType },

    /// Pops a value from the stack, and generally **stores it in a variable**,
    /// array, or otherwise.
    ///
    /// Generally, `type2` signifies the type of the value on the stack (the one
    /// to pop) and `type1` will be [`DataType::Variable`].
    /// However, there are exceptions to this. For example, when the variable
    /// reference mode is not [`ReferenceType::Normal`], then `type1` may be
    /// [`DataType::Int32`] instead.
    ///
    /// There is an alternate instruction/mode with the same opcode that swaps
    /// values around on the stack. This operation is known as
    /// [`Instruction::PopSwap`]. Note that this does not apply to this enum
    /// variant ([`Instruction::Pop`]).
    Pop { variable: CodeVariable, type1: DataType, type2: DataType },

    /// **Swaps** values around on the stack.
    ///
    /// This instruction has the same opcode as [`Instruction::Push`].
    ///
    /// For "normal mode" (`is_array = false`), it does the following:
    /// 1) Pops value A
    /// 2) Pops value B
    /// 3) Pops value C
    /// 4) Pushes value A
    /// 5) Pushes value C
    /// 6) Pushes value B
    ///
    /// This essentially turns the three stacktop
    /// elements [A, B, C, ...] into [B, C, A, ...].
    ///
    /// For array mode, it does the following:
    /// 1) Pops value A
    /// 2) Pops value B
    /// 3) Pops value C
    /// 4) Pops value D
    /// 5) Pushes value A
    /// 6) Pushes value D
    /// 7) Pushes value C
    /// 8) Pushes value B
    ///
    /// This essentially turns four stacktop elements
    /// [A, B, C, D, ...] into [B, C, D, A, ...].
    PopSwap { is_array: bool },

    /// **Duplicates** `count + 1` values on the stack.
    ///
    /// Example:
    /// Say this is your stack, top to bottom, all [`DataType::Int32`]:
    /// * 1
    /// * 2
    /// * 3
    ///
    /// The instruction `dup.i 1` will transform it into this:
    /// * 1
    /// * 2
    /// * 1
    /// * 2
    /// * 3
    ///
    /// Usually, this instruction is used to duplicate homogenous data types,
    /// and the specified `data_type` actually matches the types on the stack.
    /// However, this does not need to be the case.
    /// This instruction simply copies `(count + 1) * data_type.size_bytes()` bytes.
    /// You could use a `dup.l` in place of a `dup.d` and nothing would change.
    /// You could use a `dup.v 0` to duplicate 4 integers.
    /// You could use a `dup.l 2` to duplicate a variable and a double.
    Duplicate { data_type: DataType, count: u8 },

    /// **Swaps** values around on the stack.
    ///
    /// First, elements with a total size of `size1` are popped into a temporary
    /// "top stack". Then, elements with a total size of `size2` are popped
    /// into a temporary "bottom stack". Afterward, the "bottom stack" is
    /// pushed. And lastly, the "top stack" is pushed.
    ///
    /// For information on the data type, see [`Instruction::Duplicate`].
    DuplicateSwap { data_type: DataType, size1: u8, size2: u8 },

    /// **Exits** the current code entry with no return value.
    ///
    /// The engine still pushes a value `0` with [`DataType::Variable`] to the stack
    /// (otherwise, you could corrupt the stack in high-level GML).
    /// This implicit return value of zero also occurs when the code entry naturally exits
    /// by reaching the end of the instructions.
    ///
    /// This instruction always has the data type [`DataType::Int32`] for
    /// whatever reason. Since this data type carries no meaningful
    /// information, it is not stored in this enum variant.
    Exit,

    /// Pops a value from the stack, and **returns** from the current
    /// code entry with that value as the return value.
    ///
    /// This only makes sense to use in scripts.
    ///
    /// This instruction always has the data type [`DataType::Variable`],
    /// which is omitted from this enum data since it is redundant.
    /// However, you may still need to convert your stack value to
    /// [`DataType::Variable`] using [`Instruction::Convert`] before returning.
    Return,

    /// **Pops** a value from the stack, and **discards** it.
    ///
    /// This is similar to [`Instruction::Pop`], except it
    /// does not store the result in any variable.
    ///
    /// This instruction is commonly used to clean up unused return values of function calls.
    PopDiscard { data_type: DataType },

    /// Unconditionally **branches** (jumps) to another instruction in the code entry.
    ///
    /// Also known as `B`, `br`, `Jump`, `jmp`.
    ///
    /// The jump offset may be negative and is expressed in multiples of 4
    /// bytes. For example, a jump offset of 2 may skip `push.s`, a jump
    /// offset of 5 may skip `push.d`. Most of the time, this will skip
    /// multiple instructions at the same time.
    Branch { jump_offset: i32 },

    /// Pops a boolean/int32 value from the stack.
    /// If true/nonzero, **branches** (jumps) to another instruction in the code
    /// entry.
    ///
    /// Also known as `BranchTrue`, `bt`.
    ///
    /// The jump offset is explained in [`Instruction::Branch`].
    BranchIf { jump_offset: i32 },

    /// Pops a boolean/int32 value from the stack.
    /// If false/zero, **branches** (jumps) to another instruction in the code
    /// entry.
    ///
    /// Also known as `BranchFalse`, `bf`.
    ///
    /// The jump offset is explained in [`Instruction::Branch`].
    BranchUnless { jump_offset: i32 },

    /// Pushes a `with` context* used for GML `with` statements,
    /// to the VM environment/self instance stack.
    ///
    /// This does not push any value to the value stack (like
    /// [`Instruction::Push`]). It is rather classified as branch
    /// instruction. The specified jump offset will be branched to when the
    /// `with` loop is done. The branch target leads to the code after the
    /// `with` block.
    ///
    /// The jump offset is further explained in [`Instruction::Branch`].
    PushWithContext { jump_offset: i32 },

    /// Pops/ends a `with` context from the VM environment/self instance stack.
    ///
    /// This instruction will branch to its encoded address until no longer
    /// iterating instances, where the context will finally be gone for good.
    ///
    /// There is a different mode for this instruction with the same opcode:
    /// [`Instruction::PopWithContextExit`].
    ///
    /// The jump offset is further explained in [`Instruction::Branch`].
    PopWithContext { jump_offset: i32 },

    /// A variation of [`Instruction::PopWithContext`].
    ///
    /// This variation exits the `with` loop context without branching anywhere.
    /// Since the instruction pointer is malformed afterward, this instruction
    /// is only seen before a [`Instruction::Exit`],
    /// other [`Instruction::PopWithContextExit`]s or perhaps
    /// [`Instruction::PopDiscard`].
    PopWithContextExit,

    /// **Pushes** a constant value onto the stack.
    /// Can vary in size depending on value type.
    ///
    /// This instruction can also push variables (which copies the value),
    /// but I don't know the exact behavior / internal implementation.
    ///
    /// DOCME: Explain pushing of variables better
    Push { value: PushValue },

    /// Pushes a value stored in a local variable onto the stack.
    ///
    /// This is a specialization of the [`Instruction::Push`] instruction
    /// where `value` is [`PushValue::Variable`] whose instance type is
    /// [`InstanceType::Local`].
    ///
    /// This is only a minor optimization; using the standard push instruction
    /// also works fine.
    PushLocal { variable: CodeVariable },

    /// Pushes a value stored in a global variable onto the stack.
    ///
    /// This is a specialization of the [`Instruction::Push`] instruction
    /// where `value` is [`PushValue::Variable`] whose instance type is
    /// [`InstanceType::Global`].
    ///
    /// This is only a minor optimization; using the standard push instruction
    /// also works fine.
    PushGlobal { variable: CodeVariable },

    /// Pushes a value stored in a GameMaker builtin variable onto the stack.
    ///
    /// This is a specialization of the [`Instruction::Push`] instruction
    /// where `value` is [`PushValue::Variable`] whose instance type is
    /// [`InstanceType::Builtin`].
    ///
    /// This is only a minor optimization; using the standard push instruction
    /// also works fine.
    PushBuiltin { variable: CodeVariable },

    /// Pushes an immediate signed 32-bit integer value onto the stack, encoded
    /// as a signed 16-bit integer.
    ///
    /// The data type of this instruction is always [`DataType::Int16`],
    /// which is not stored here to avoid redundancy.
    ///
    /// Please note that [`DataType::Int16`] is only a valid data type in
    /// instructions when pushing. Using it anywhere else is wrong, because
    /// `Int16`s immediately get converted to `Int32`s when pushed on the
    /// stack. The data type `Int16` *does not exist* on the stack.
    PushImmediate { integer: i16 },

    /// Calls a GML script or builtin function.
    ///
    /// Arguments should be pushed prior to this instruction, in reverse order.
    /// Make sure to convert them to [`DataType::Variable`] as well.
    ///
    /// # Return
    /// After the call is complete, a return value of [`DataType::Variable`] will be
    /// pushed to the stack. This always occurs; even if the called script exited without `return`.
    /// In that case, the return value will be `undefined` (or `0` in pre-WAD15).
    /// To ignore the return value of a call, you can use [`Self::PopDiscard`].
    ///
    /// # Arguments
    /// Function calls are allowed to have an arbitary number of arguments.
    ///
    /// Builtin functions will throw an error if arguments are missing (`arg_count` too low).
    /// They do not throw an error or misbehave if you provide too many arguments.
    /// Certain builtin functions are designed to handle any number of arguments, such as `ds_list_add`.
    ///
    /// For scripts, the pushed arguments are placed into the special `argument0` - `argument15` variables.
    /// Accessing an uninitialized argument variable yields `undefined` (or `0` in pre-WAD15).
    /// Arguments are never initialized in events/room creation code; only in scripts.
    Call { function: GMRef<Function>, arg_count: u16 },

    /// Pops two values off of the stack, and then calls a
    /// GML script/function using those values, representing
    /// the "self" instance to be used when calling,
    /// as well as the reference to the function being called.
    ///
    /// This instruction pops two values off the stack and then calls a
    /// function, dynamically:
    /// 1) The function reference is popped (should be a [`DataType::Variable`]
    ///    value storing a function ID).
    /// 2) The instance type is popped. I'm not very sure how this works, but I
    ///    assume it is a raw [`InstanceType`] value stored in the stack?
    /// 3) `argument_count` arguments are popped.
    ///
    /// For more information on calling functions, see [`Instruction::Call`].
    CallVariable { arg_count: u16 },

    /// Verifies an array index is within proper
    /// bounds, typically for multidimensional arrays.
    ///
    /// DOCME: How does this work? What does it actually do?
    CheckArrayIndex,

    /// Pops two values from the stack, those being an index and an array
    /// reference. Then, pushes the value stored at the passed-in array at
    /// the desired index.
    ///
    /// This is a very similar to [`Instruction::PushArrayContainer`] (see this
    /// for more info), except that this instruction is used only at the end
    /// of an accessor chain. Only relevant for the final/last index
    /// operation of a multidimensional array access.
    PushArrayFinal,

    /// Pops three values from the stack, those being an index, an array
    /// reference, and a value. Then, assigns the value to the array at the
    /// specified index.
    ///
    /// Concrete Steps:
    /// 1) Pops an index from the stack
    /// 2) Pops array reference from stack ([`DataType::Variable`])
    /// 3) Pops value from stack and moves it into the specified array item
    PopArrayFinal,

    /// Pushes a multidimensional array:
    /// 1) Pops an index from the stack
    /// 2) Pops array reference from stack ([`DataType::Variable`])
    /// 3) Pushes a new array reference from the passed-in array at the desired
    ///    index
    ///
    /// This instruction is used for all multidimensional
    /// index pushes from the second through the second to last.
    /// The final/last index operation will be done using
    /// [`Instruction::PushArrayFinal`].
    PushArrayContainer,

    /// Sets a global variable in the VM (popped from stack), designated for
    /// tracking the now-deprecated array copy-on-write functionality in GML.
    ///
    /// The value used is specific to certain locations in scripts.
    /// When array copy-on-write functionality is disabled, this extended opcode
    /// is not used.
    ///
    /// This instruction will pop one value (`Int32`) off the stack, indicating
    /// the array owner ID.
    SetArrayOwner,

    /// Pushes a boolean value to the stack, indicating whether static
    /// initialization has already occurred for this function (true), or
    /// otherwise false.
    ///
    /// This is typically used in conjunction with
    /// [`Instruction::SetStaticInitialized`] and branch instructions.
    HasStaticInitialized,

    /// Marks the current function to no longer be able to enter its own static
    /// initialization.
    ///
    /// This can either occur at the beginning or end of a static block,
    /// depending on whether `AllowReentrantStatic` is enabled by a game's
    /// developer (enabled by default before GameMaker 2024.11; disabled by
    /// default otherwise).
    ///
    /// This is typically used in conjunction with
    /// [`Instruction::HasStaticInitialized`] and branch instructions.
    SetStaticInitialized,

    /// Keeps track of an array reference temporarily.
    /// Used in multidimensional array compound assignment statements
    /// ([`Instruction::PushArrayFinal`] etc.).
    ///
    /// Presumed to be used for garbage collection purposes.
    SaveArrayReference,

    /// Restores a previously-tracked array reference.
    /// Used in multidimensional array compound assignment statements
    /// ([`Instruction::PushArrayFinal`] etc.).
    ///
    /// Presumed to be used for garbage collection purposes.
    RestoreArrayReference,

    /// Pops a value from the stack, and pushes a boolean result.
    /// The result is true if a "nullish" value, such as `undefined` or GML's
    /// `pointer_null`.
    IsNullishValue,

    /// Pushes an asset reference to the stack, encoded in an integer.
    /// Includes asset type and index.
    ///
    /// This instruction is preferred over normal [`Instruction::Push`] with
    /// [`DataType::Int32`], since the intent is clearer that this is an
    /// encoded asset reference, not an actual integer.
    ///
    /// This instruction is used in more modern versions of GameMaker.
    PushReference { asset_reference: AssetReference },
}

impl Instruction {
    /// Gets the instruction size in multiples of 4 bytes.
    ///
    /// This unit is used by `jump_offset`s in branch instructions.
    ///
    /// This size includes extra data like integers, floats, variable references, etc.
    /// For example, a [`Instruction::Push`] with a [`PushValue::Double`] has a size of 3.
    #[must_use]
    pub const fn size(&self) -> u32 {
        match self {
            Self::Push { value } => match value {
                PushValue::Short(_) => 1,
                PushValue::Long(_) | PushValue::Double(_) => 3,
                _ => 2,
            },
            Self::Pop { .. }
            | Self::PushLocal { .. }
            | Self::PushGlobal { .. }
            | Self::PushBuiltin { .. }
            | Self::Call { .. }
            | Self::PushReference { .. } => 2,
            _ => 1,
        }
    }

    /// Gets the instruction size in bytes.
    ///
    /// This size includes extra data like integers, floats, variable references, etc.
    /// For example, a [`Instruction::Push`] with a [`PushValue::Double`] has a size of 12 bytes.
    #[must_use]
    pub const fn size_bytes(&self) -> u32 {
        self.size() * 4
    }

    /// Attempts to extract a [`CodeVariable`] from the instruction.
    ///
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

    /// Attempts to extract a [`CodeVariable`] from the instruction.
    ///
    /// This can succeed for `Push` and will always succeed for
    /// `PushGlobal`, `PushLocal`, `PushBuiltin` and `Pop`.
    #[must_use]
    pub const fn variable_mut(&mut self) -> Option<&mut CodeVariable> {
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
    ///
    /// This can succeed for `Push` and `PushReference` and will always succeed
    /// for `Call`.
    #[must_use]
    pub const fn function(&self) -> Option<GMRef<Function>> {
        match self {
            Self::Push { value: PushValue::Function(function) }
            | Self::Call { function, .. }
            | Self::PushReference { asset_reference: AssetReference::Function(function) } => {
                Some(*function)
            }
            _ => None,
        }
    }

    /// Attempts to extract a `GMRef<GMFunction>` from the instruction.
    ///
    /// This can succeed for `Push` and `PushReference` and will always succeed
    /// for `Call`.
    #[must_use]
    pub const fn function_mut(&mut self) -> Option<&mut GMRef<Function>> {
        match self {
            Self::Push { value: PushValue::Function(function) }
            | Self::Call { function, .. }
            | Self::PushReference { asset_reference: AssetReference::Function(function) } => {
                Some(function)
            }
            _ => None,
        }
    }

    /// Attempts to extract a jump offset in bytes from the instruction.
    ///
    /// This will always succeed for `Branch`, `BranchIf`,
    /// `BranchUnless`, `PushWithContext` and `PopWithContext`.
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

    /// Attempts to extract a jump offset in bytes from the instruction.
    ///
    /// This will always succeed for `Branch`, `BranchIf`,
    /// `BranchUnless`, `PushWithContext` and `PopWithContext`.
    #[must_use]
    pub const fn jump_offset_mut(&mut self) -> Option<&mut i32> {
        match self {
            Self::Branch { jump_offset }
            | Self::BranchIf { jump_offset }
            | Self::BranchUnless { jump_offset }
            | Self::PushWithContext { jump_offset }
            | Self::PopWithContext { jump_offset } => Some(jump_offset),
            _ => None,
        }
    }

    /// Attempts to extract the first (or the only) data type from the
    /// instruction.
    ///
    /// For binary operations, this will be RHS (the right hand side).
    ///
    /// NOTE: Right now, it's kind of arbitrary which instructions' data types
    /// are deemed "relevant enough" to return them and which don't really
    /// belong (does return: `PushLocal` - Variable, does not return:
    /// `PopSwap` - Int16). This will be changed in the future **if I get
    /// feedback pls**.
    #[must_use]
    pub const fn type1(&self) -> Option<DataType> {
        Some(match *self {
            Self::Convert { from, .. } => from,
            Self::Binary { rhs, .. } => rhs,
            Self::Negate { data_type }
            | Self::Not { data_type }
            | Self::Duplicate { data_type, .. }
            | Self::DuplicateSwap { data_type, .. }
            | Self::PopDiscard { data_type } => data_type,
            Self::Pop { type1, .. } => type1,
            Self::Push { value } => value.data_type(),
            Self::PushLocal { .. } | Self::PushGlobal { .. } | Self::PushBuiltin { .. } => {
                DataType::Variable
            }
            Self::PushImmediate { .. } => DataType::Short,
            Self::Call { .. } => DataType::Int,
            Self::CallVariable { .. } => DataType::Variable,
            _ => return None,
        })
    }

    /// Attempts to return the second data type of this instruction.
    ///
    /// For binary operations, this will be LHS (the left hand side).
    #[must_use]
    pub const fn type2(&self) -> Option<DataType> {
        Some(match *self {
            Self::Convert { to, .. } => to,
            Self::Binary { lhs, .. } => lhs,
            Self::Pop { type2, .. } => type2,
            _ => return None,
        })
    }
}
