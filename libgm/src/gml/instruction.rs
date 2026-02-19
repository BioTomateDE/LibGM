//! Everything related to GML instructions is in here,
//! including the important [`Instruction`] type.

mod asset_reference;
mod category;
mod code_variable;
mod comparison_type;
mod data_type;
mod instance_type;
mod push_value;
mod variable_type;

pub use asset_reference::AssetReference;
pub use category::Category;
pub use code_variable::CodeVariable;
pub use comparison_type::ComparisonType;
pub use data_type::DataType;
pub use instance_type::InstanceType;
pub use push_value::PushValue;
pub use variable_type::VariableType;

use crate::gamemaker::{elements::function::GMFunction, reference::GMRef};

/// A GameMaker VM Instruction.
///
/// This is the most important data type for GML.
///
/// For more information on GML, see the [module level documentation].
///
/// [module level documentation]: crate::gml
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    /// Converts the top of the stack from one type to another.
    ///
    /// Sometimes it may be necessary to convert between "actual data types" and [`DataType::Variable`]
    /// (note: not sure if necessary, but the YoYoGames compiler generates it).
    /// For example, when calling a function, all arguments need (?) to of data type [`DataType::Variable`].
    /// So if you want to call `foo(41)`:
    /// ```
    /// pushim 41
    /// conv.i.v
    /// call foo(argc=1)
    /// ```
    Convert { from: DataType, to: DataType },

    /// Pops two values from the stack, **multiplies** them, and pushes the result.
    Multiply {
        multiplicand: DataType,
        multiplier: DataType,
    },

    /// Pops two values from the stack, **divides** them, and pushes the result.
    /// The second popped value (`dividend`) is divided by the first popped value (`divisor`).
    Divide {
        dividend: DataType,
        divisor: DataType,
    },

    /// Pops two values from the stack, performs a GML `div` operation (division with remainder), and pushes the result.
    /// The second popped value (`dividend`) is divided (with remainder) by the first popped value (`divisor`).
    ///
    /// This operation is similar to [`Instruction::Modulus`], except it behaves differently for negative values.
    /// For example: `-19 rem 12 == -7` (not 5).
    Remainder {
        dividend: DataType,
        divisor: DataType,
    },

    /// Pops two values from the stack, performs a GML `mod` operation (`%`), and pushes the result.
    /// The second popped value is modulo'd against the first popped value.
    ///
    /// This operation is similar to [`Instruction::Remainder`], except it behaves differently for negative values.
    /// This `modulus` operation performs [Euclidean division](https://en.wikipedia.org/wiki/Euclidean_division).
    /// For example: `-19 rem 12 == 5` (not -7).
    Modulus {
        dividend: DataType,
        divisor: DataType,
    },

    /// Pops two values from the stack, **adds** them, and pushes the result.
    Add { augend: DataType, addend: DataType },

    /// Pops two values from the stack, **subtracts** them, and pushes the result.
    /// The second popped value is subtracted by the first popped value.
    Subtract {
        minuend: DataType,
        subtrahend: DataType,
    },

    /// Pops two values from the stack, performs an **AND** operation, and pushes the result.
    /// This can be done bitwise or logically, depending on the data type(s).
    And { lhs: DataType, rhs: DataType },

    /// Pops two values from the stack, performs an **OR** operation, and pushes the result.
    /// This can be done bitwise or logically, depending on the data type(s).
    Or { lhs: DataType, rhs: DataType },

    /// Pops two values from the stack, performs an **XOR** operation, and pushes the result.
    /// This can be done bitwise or logically, depending on the data type(s).
    Xor { lhs: DataType, rhs: DataType },

    /// **Negates** the top value of the stack (as in, multiplies it with -1).
    Negate { data_type: DataType },

    /// Pops one value from the stack, performs a **NOT** operation, and pushes the result.
    /// This can be done bitwise or logically, depending on the data type(s).
    Not { data_type: DataType },

    /// Pops two values from the stack, performs a bitwise **left shift** operation (`<<`), and pushes the result.
    /// The second popped value (`value`) is shifted left by the first popped value (`shift_amount`).
    ShiftLeft {
        value: DataType,
        shift_amount: DataType,
    },

    /// Pops two values from the stack, performs a bitwise **right shift** operation (`>>`), and pushes the result.
    /// The second popped value (`value`) is shifted right by the first popped value (`shift_amount`).
    ShiftRight {
        value: DataType,
        shift_amount: DataType,
    },

    /// Pops two values from the stack, **compares** them using a [`ComparisonType`],
    /// and pushes a boolean result ([`DataType::Boolean`]).
    Compare {
        lhs: DataType,
        rhs: DataType,
        comparison_type: ComparisonType,
    },

    /// Pops a value from the stack, and generally **stores it in a variable**, array, or otherwise.
    ///
    /// Generally, `type1` signifies the type of the value on the stack (the one to pop)
    /// and `type2` will be [`DataType::Variable`].
    /// However, there are exceptions to this. For example, when the variable reference mode is not [`VariableType::Normal`],
    /// then `type2` may be [`DataType::Int32`] instead.
    ///
    /// There is an alternate instruction/mode with the same opcode that swaps values around on the stack.
    /// This operation is known as [`Instruction::PopSwap`].
    /// Note that this does not apply to this enum variant ([`Instruction::Pop`].
    Pop {
        variable: CodeVariable,
        type1: DataType,
        type2: DataType,
    },

    /// **Swaps** values around on the stack.
    ///
    /// This instruction has the same opcode as [`Instruction::Push`].
    ///
    /// TODO(doc): explain concrete behavior
    PopSwap { is_array: bool },

    /// **Duplicates** values on the stack.
    ///
    /// The specified `size` is the total size of all
    /// elements that should be duplicated.
    ///
    /// The specified data type hints what the stacktop data type is.
    /// However, it does (most likely?) not explicitly have to match.
    /// It only influences a multiplication factor of how many bytes to
    /// clone, since different data types have different sizes on the stack.
    Duplicate { data_type: DataType, size: u8 },

    /// **Swaps** values around on the stack.
    ///
    /// First, elements with a total size of `size1` are popped into a temporary "top stack".
    /// Then, elements with a total size of `size2` are popped into a temporary "bottom stack".
    /// Afterward, the "bottom stack" is pushed.
    /// And lastly, the "top stack" is pushed.
    ///
    /// For information on the data type, see [`Instruction::Duplicate`].
    DuplicateSwap {
        data_type: DataType,
        size1: u8,
        size2: u8,
    },

    /// Pops a value from the stack, and **returns** from the current
    /// function/script with that value as the return value.
    ///
    /// This instruction always has the data type [`DataType::Variable`],
    /// which is omitted from this enum data since it is redundant.
    /// However, you may still need to convert your stack value to
    /// [`DataType::Variable`] using [`Instruction::Convert`] before returning.
    Return,

    /// **Returns** from the current function/script/event with no return value.
    ///
    /// This instruction always has the data type [`DataType::Int32`] for whatever reason.
    /// Since this data type carries no meaningful information, it is not stored in this enum variant.
    Exit,

    /// **Pops** a value from the stack, and **discards** it.
    ///
    /// This is similar to [`Instruction::Pop`], except it does not store the result in any variable.
    ///
    /// This instruction is commonly used to clean up unused return values of function calls.
    PopDiscard { data_type: DataType },

    /// Unconditionally **branches** (jumps) to another instruction in the code entry.
    ///
    /// Also known as `B`, `Jump`, `jmp`.
    ///
    /// The jump offset may be negative and is expressed in multiples of 4 bytes.
    /// For example, a jump offset of 2 may skip `push.s`, a jump offset of 5 may skip `push.d`.
    /// Most of the time, this will skip multiple instructions at the same time.
    Branch { jump_offset: i32 },

    /// Pops a boolean/int32 value from the stack.
    /// If true/nonzero, **branches** (jumps) to another instruction in the code entry.
    ///
    /// Also known as `BranchTrue`, `bt`.
    ///
    /// The jump offset is explained in [`Instruction::Branch`].
    BranchIf { jump_offset: i32 },

    /// Pops a boolean/int32 value from the stack.
    /// If false/zero, **branches** (jumps) to another instruction in the code entry.
    ///
    /// Also known as `BranchFalse`, `bf`.
    ///
    /// The jump offset is explained in [`Instruction::Branch`].
    BranchUnless { jump_offset: i32 },

    /// Pushes a `with` context* used for GML `with` statements,
    /// to the VM environment/self instance stack.
    ///
    /// This does not push any value to the value stack (like [`Instruction::Push`]).
    /// It is rather classified as branch instruction.
    /// The specified jump offset will be branched to when the `with` loop is done.
    /// The branch target leads to the code after the `with` block.
    ///
    /// The jump offset is further explained in [`Instruction::Branch`].
    PushWithContext { jump_offset: i32 },

    /// Pops/ends a `with` context, used for GML `with` statements,
    /// from the VM environment/self instance stack.
    /// This instruction will branch to its encoded address until no longer
    /// iterating instances, where the context will finally be gone for good.
    ///
    /// There is a different mode for this instruction with the same opcode: [`Instruction::PopWithContextExit`].
    ///
    /// The jump offset is further explained in [`Instruction::Branch`].
    PopWithContext { jump_offset: i32 },

    /// A variation of [`Instruction::PopWithContext`].
    /// This variation exits the `with` loop context without branching anywhere.
    /// Since the instruction pointer is malformed afterward, this instruction
    /// is only seen before a [`Instruction::Exit`],
    /// other [`Instruction::PopWithContextExit`]s or perhaps [`Instruction::PopDiscard`].
    PopWithContextExit,

    /// **Pushes** a constant value onto the stack.
    /// Can vary in size depending on value type.
    ///
    /// This instruction can also push variables (which copies the value),
    /// but I don't know the exact behavior / internal implementation.
    ///
    /// TODO(doc): Explain pushing of variables better
    Push { value: PushValue },

    /// Pushes a value stored in a local variable onto the stack.
    ///
    /// This is a specialization of the [`Instruction::Push`] instruction
    /// where `value` is [`PushValue::Variable`] whose instance type is [`InstanceType::Local`].
    ///
    /// This is only a minor optimization; using the standard push instruction also works fine.
    PushLocal { variable: CodeVariable },

    /// Pushes a value stored in a global variable onto the stack.
    ///
    /// This is a specialization of the [`Instruction::Push`] instruction
    /// where `value` is [`PushValue::Variable`] whose instance type is [`InstanceType::Global`].
    ///
    /// This is only a minor optimization; using the standard push instruction also works fine.
    PushGlobal { variable: CodeVariable },

    /// Pushes a value stored in a GameMaker builtin variable onto the stack.
    ///
    /// This is a specialization of the [`Instruction::Push`] instruction
    /// where `value` is [`PushValue::Variable`] whose instance type is [`InstanceType::Builtin`].
    ///
    /// This is only a minor optimization; using the standard push instruction also works fine.
    PushBuiltin { variable: CodeVariable },

    /// Pushes an immediate signed 32-bit integer value onto the stack, encoded as a signed 16-bit integer.
    ///
    /// The data type of this instruction is always [`DataType::Int16`],
    /// which is not stored here to avoid redundancy.
    ///
    /// Please note that [`DataType::Int16`] is only a valid data type in instructions when pushing.
    /// Using it anywhere else is wrong, because `Int16`s
    /// immediately get converted to `Int32`s when pushed on the stack.
    /// The data type `Int16` *does not exist* on the stack.
    PushImmediate { integer: i16 },

    /// Calls a GML script/function, using its ID (index).
    /// Arguments are prepared prior to this instruction, in reverse order.
    ///
    /// Argument count is encoded in this instruction.
    /// Arguments are popped off of the stack.
    ///
    /// Every function call is allowed to have an arbitary number of arguments.
    /// Certain builtin functions are designed to handle any
    /// number of arguments, such as `ds_list_add`.
    /// For custom functions (aka. scripts), the remaining values will be filled
    /// with specified default values or `undefined` (?).
    /// TODO(doc): I'm not sure what happens in WAD<15.
    /// TODO(doc): I'm not sure what happens when too many arguments are specified (probably nothing?).
    Call {
        function: GMRef<GMFunction>,
        argument_count: u16,
    },

    /// Pops two values off of the stack, and then calls a
    /// GML script/function using those values, representing
    /// the "self" instance to be used when calling,
    /// as well as the reference to the function being called.
    ///
    /// This instruction pops two values off the stack and then calls a function, dynamically:
    /// 1) The function reference is popped
    ///    should be a [`DataType::Variable`] value storing a function ID).
    /// 2) The instance type is popped. I'm not very sure how this works, but I  assume it
    ///    is a raw [`InstanceType`] value stored in the stack?
    /// 3) `argument_count` arguments are popped.
    ///
    /// For more information on calling functions, see [`Instruction::Call].
    CallVariable { argument_count: u16 },

    /// Verifies an array index is within proper
    /// bounds, typically for multidimensional arrays.
    ///
    /// TODO(doc): How does this work? What does it actually do?
    CheckArrayIndex,

    /// Pops two values from the stack, those being an index and an array reference.
    /// Then, pushes the value stored at the passed-in array at the desired index.
    ///
    /// This is a very similar to [`Instruction::PushArrayContainer`],
    /// except that this instruction is used only at the end of an accessor chain.
    /// Only relevant for the final/last index operation of a multidimensional array access.
    PushArrayFinal,

    /// Pops three values from the stack, those being an index, an array reference, and a value.
    /// Then, assigns the value to the array at the specified index.
    PopArrayFinal,

    /// Pushes a multidimensional array:
    /// 1) Pops an index from the stack
    /// 2) Pops array reference from stack ([`DataType::Variable`])
    /// 3) Pushes a new array reference from the passed-in array at the desired index
    ///
    /// This instruction is used for for all multidimensional
    /// index pushes from the second through the second to last.
    /// The final/last index operation will be done using [`Instruction::PushArrayFinal`].
    PushArrayContainer,

    /// Sets a global variable in the VM (popped from stack), designated for
    /// tracking the now-deprecated array copy-on-write functionality in GML.
    ///
    /// The value used is specific to certain locations in scripts.
    /// When array copy-on-write functionality is disabled, this extended opcode is not used.
    ///
    /// This instruction will pop one value (`Int32`) off the stack, indicating the array owner ID.
    SetArrayOwner,

    /// Pushes a boolean value to the stack, indicating whether static initialization
    /// has already occurred for this function (true), or otherwise false.
    ///
    /// This is typically used in conjuntion with [`Instruction::SetStaticInitialized`] and branch instructions.
    HasStaticInitialized,

    /// Marks the current function to no longer be able to enter its own static initialization.
    ///
    /// This can either occur at the beginning or end of a static block,
    /// depending on whether `AllowReentrantStatic` is enabled by a game's developer
    /// (enabled by default before GameMaker 2024.11; disabled by default otherwise).
    ///
    /// This is typically used in conjuntion with [`Instruction::HasStaticInitialized`] and branch instructions.
    SetStaticInitialized,

    /// Keeps track of an array reference temporarily.
    /// Used in multidimensional array compound assignment statements
    /// ([`Instruction::PushArrayFinal`] etc).
    ///
    /// Presumed to be used for garbage collection purposes.
    SaveArrayReference,

    /// Restores a previously-tracked array reference.
    /// Used in multidimensional array compound assignment statements
    /// ([`Instruction::PushArrayFinal`] etc).
    ///
    /// Presumed to be used for garbage collection purposes.
    RestoreArrayReference,

    /// Pops a value from the stack, and pushes a boolean result.
    /// The result is true if a "nullish" value, such as `undefined` or GML's `pointer_null`.
    IsNullishValue,

    /// Pushes an asset reference to the stack, encoded in an integer.
    /// Includes asset type and index.
    ///
    /// This instruction is preferred over normal [`Instruction::Push`] with [`DataType::Int32`],
    /// since the intent is clearer that this is an encoded asset reference, not an actual integer.
    ///
    /// This instruction is used in more modern versions of GameMaker.
    PushReference { asset_reference: AssetReference },
}

impl Instruction {
    /// Gets the instruction size in multiples of 4 bytes.
    /// This unit is used by `jump_offset`s in branch instructions.
    ///
    /// For example, [`Instruction::Push`] with a [`PushValue::Int16`] has a size of 5.
    #[must_use]
    pub const fn size4(&self) -> u8 {
        match self {
            Self::Push { value } => match value {
                PushValue::Int16(_) => 1,
                PushValue::Int64(_) | PushValue::Double(_) => 3,
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
    /// This size includes extra data like integers, floats, variable references, etc.
    ///
    /// For example, [`Instruction::Push`] with a [`PushValue::Int16`] has a size of 20.
    #[must_use]
    pub const fn size(&self) -> u8 {
        // TODO(break): change type to u32 in next semver major
        self.size4() * 4
    }

    /// Attempts to extract a [`CodeVariable`] from  the instruction.
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

    /// Attempts to extract a `GMRef<GMFunction>` from the instruction.
    ///
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

    /// Attempts to extract the first (or the only) data type from the instruction.
    ///
    /// For binary operations, this will be RHS (the right hand side).
    ///
    /// NOTE: Right now, it's kind of arbitary which instructions' data types are
    /// deemed "relevant enough" to return them and which don't really belong
    /// (does return: `PushLocal` - Variable, does not return: `PopSwap` - Int16).
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
    /// For binary operations, this will be LHS (the left hand side).
    #[must_use]
    pub const fn type2(&self) -> Option<DataType> {
        Some(match *self {
            Self::Convert { to, .. } => to,
            Self::Multiply { multiplicand, .. } => multiplicand,
            Self::Divide { dividend, .. }
            | Self::Remainder { dividend, .. }
            | Self::Modulus { dividend, .. } => dividend,
            Self::Add { augend, .. } => augend,
            Self::Subtract { minuend, .. } => minuend,
            Self::And { lhs, .. }
            | Self::Or { lhs, .. }
            | Self::Xor { lhs, .. }
            | Self::Compare { lhs, .. } => lhs,
            Self::ShiftLeft { value, .. } | Self::ShiftRight { value, .. } => value,
            Self::Pop { type2, .. } => type2,
            _ => return None,
        })
    }
}
