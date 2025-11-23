use std::fmt::{Display, Formatter};

use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::gamemaker::{
    elements::{
        animation_curves::GMAnimationCurve, backgrounds::GMBackground, fonts::GMFont,
        functions::GMFunction, game_objects::GMGameObject, particle_systems::GMParticleSystem,
        paths::GMPath, rooms::GMRoom, scripts::GMScript, sequence::GMSequence, shaders::GMShader,
        sounds::GMSound, sprites::GMSprite, timelines::GMTimeline, variables::GMVariable,
    },
    reference::GMRef,
};

/// A code entry in a data file.
#[derive(Debug, Clone, PartialEq)]
pub struct GMCode {
    /// The name of the code entry.
    pub name: String,

    /// A list of bytecode instructions this code entry has.
    pub instructions: Vec<GMInstruction>,

    /// Set in bytecode 15+.
    pub bytecode15_info: Option<GMCodeBytecode15>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMCodeBytecode15 {
    /// The amount of local variables this code entry has.
    pub locals_count: u16,

    /// The amount of arguments this code entry accepts.
    pub arguments_count: u16,

    /// A flag set on certain code entries, which usually don't have locals attached to them.
    pub weird_local_flag: bool,

    /// Offset, **in bytes**, where code should begin executing from within the bytecode of this code entry.
    /// Should be 0 for root-level (parent) code entries, and nonzero for child code entries.
    pub offset: u32,

    /// Parent entry of this code entry, if this is a child entry; [`None`] otherwise.
    pub parent: Option<GMRef<GMCode>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GMInstruction {
    /// Converts the top of the stack from one type to another.
    Convert { from: GMDataType, to: GMDataType },

    /// Pops two values from the stack, multiplies them, and pushes the result.
    //TODO: make sure these lhs rhs are labelled correctly
    Multiply {
        multiplicand: GMDataType,
        multiplier: GMDataType,
    },

    /// Pops two values from the stack, divides them, and pushes the result.
    /// The second popped value is divided by the first popped value.
    Divide {
        dividend: GMDataType,
        divisor: GMDataType,
    },

    /// Pops two values from the stack, performs a GML `div` operation (division with remainder), and pushes the result.
    /// The second popped value is divided (with remainder) by the first popped value.
    Remainder {
        dividend: GMDataType,
        divisor: GMDataType,
    },

    /// Pops two values from the stack, performs a GML `mod` operation (`%`), and pushes the result.
    /// The second popped value is modulo'd against the first popped value.
    Modulus {
        dividend: GMDataType,
        divisor: GMDataType,
    },

    /// Pops two values from the stack, adds them, and pushes the result.
    Add {
        augend: GMDataType,
        addend: GMDataType,
    },

    /// Pops two values from the stack, **subtracts** them, and pushes the result.
    /// The second popped value is subtracted by the first popped value.
    Subtract {
        minuend: GMDataType,
        subtrahend: GMDataType,
    },

    /// Pops two values from the stack, performs an **AND** operation, and pushes the result.
    /// This can be done bitwise or logically.
    And { lhs: GMDataType, rhs: GMDataType },

    /// Pops two values from the stack, performs an **OR** operation, and pushes the result.
    /// This can be done bitwise or logically.
    Or { lhs: GMDataType, rhs: GMDataType },

    /// Pops two values from the stack, performs an **XOR** operation, and pushes the result.
    /// This can be done bitwise or logically.
    Xor { lhs: GMDataType, rhs: GMDataType },

    /// Negates the top value of the stack (as in, multiplies it with negative one).
    Negate { data_type: GMDataType },

    /// Performs a boolean or bitwise NOT operation on the top value of the stack (modifying it).
    Not { data_type: GMDataType },

    /// Pops two values from the stack, performs a bitwise left shift operation (`<<`), and pushes the result.
    /// The second popped value is shifted left by the first popped value.
    ShiftLeft {
        value: GMDataType,
        shift_amount: GMDataType,
    },

    /// Pops two values from the stack, performs a bitwise right shift operation (`>>`), and pushes the result.
    /// The second popped value is shifted right by the first popped value.
    ShiftRight {
        value: GMDataType,
        shift_amount: GMDataType,
    },

    /// Pops two values from the stack, compares them using a [`GMComparisonType`], and pushes a boolean result.
    Compare {
        lhs: GMDataType,
        rhs: GMDataType,
        comparison_type: GMComparisonType,
    },

    /// Pops a value from the stack, and generally stores it in a variable, array, or otherwise.
    /// Has an alternate mode that can swap values around on the stack.
    /// TODO: type1 and type2 are bad names and are probably redundant values
    Pop {
        variable: CodeVariable,
        type1: GMDataType,
        type2: GMDataType,
    },

    /// Swaps values around on the stack
    PopSwap { is_array: bool },

    /// Duplicates values on the stack.
    ///
    /// The specified `size` is the total size of all
    /// elements that should be duplicated.
    Duplicate { data_type: GMDataType, size: u8 },

    /// Swaps values around on the stack.
    ///
    /// First, elements with a total size of `size1` are popped into a temporary "top stack".
    /// Then, elements with a total size of `size2` are popped into a temporary "bottom stack".
    /// Afterwards, the "bottom stack" is pushed.
    /// And lastly, the "top stack" is pushed.
    DuplicateSwap {
        data_type: GMDataType,
        size1: u8,
        size2: u8,
    },

    /// Pops a value from the stack, and returns from the current
    /// function/script with that value as the return value.
    Return,

    /// Returns from the current function/script/event with no return value.
    Exit,

    /// Pops a value from the stack, and discards it.
    PopDiscard { data_type: GMDataType },

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

    /// PopWithContext but with PopEnvExitMagic
    PopWithContextExit,

    /// Pushes a constant value onto the stack. Can vary in size depending on value type.
    Push { value: GMCodeValue },

    /// Pushes a value stored in a local variable onto the stack.
    PushLocal { variable: CodeVariable },

    /// Pushes a value stored in a global variable onto the stack.
    PushGlobal { variable: CodeVariable },

    /// Pushes a value stored in a `GameMaker` builtin variable onto the stack.
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
    /// TODO: got rid of datatype, its probably always `variable` right?
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
    /// depending on whether "AllowReentrantStatic" is enabled by a game's developer
    /// (enabled by default before `GameMaker` 2024.11; disabled by default otherwise).
    SetStaticInitialized,

    /// Keeps track of an array reference temporarily. Used in multidimensional array compound assignment statements.
    /// Presumed to be used for garbage collection purposes.
    SaveArrayReference,

    /// Restores a previously-tracked array reference.
    /// Used in multidimensional array compound assignment statements.
    /// Presumed to be used for garbage collection purposes.
    RestoreArrayReference,

    /// Pops a value from the stack, and pushes a boolean result.
    /// The result is true if a "nullish" value, such as undefined or GML's pointer_null.
    IsNullishValue,

    /// Pushes an asset reference to the stack, encoded in an integer. Includes asset type and index.
    PushReference { asset_reference: GMAssetReference },
}

impl GMInstruction {
    /// Gets the instruction size in bytes.
    /// This size includes extra data like integers, floats, variable references, etc.
    pub const fn size(&self) -> u32 {
        match self {
            GMInstruction::Pop { .. }
            | GMInstruction::PushLocal { .. }
            | GMInstruction::PushGlobal { .. }
            | GMInstruction::PushBuiltin { .. } => 8,
            GMInstruction::Push {
                value:
                    GMCodeValue::Int32(_)
                    | GMCodeValue::Function(_)
                    | GMCodeValue::String(_)
                    | GMCodeValue::Boolean(_),
            } => 8,
            GMInstruction::Push { value: GMCodeValue::Int64(_) | GMCodeValue::Double(_) } => 12,
            GMInstruction::Call { .. } => 8,
            GMInstruction::PushReference { .. } => 8,
            _ => 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GMAssetReference {
    Object(GMRef<GMGameObject>),
    Sprite(GMRef<GMSprite>),
    Sound(GMRef<GMSound>),
    Room(GMRef<GMRoom>),
    Background(GMRef<GMBackground>),
    Path(GMRef<GMPath>),
    Script(GMRef<GMScript>),
    Font(GMRef<GMFont>),
    Timeline(GMRef<GMTimeline>),
    Shader(GMRef<GMShader>),
    Sequence(GMRef<GMSequence>),
    AnimCurve(GMRef<GMAnimationCurve>),
    ParticleSystem(GMRef<GMParticleSystem>),
    RoomInstance(i32),
    /// Does not exist in UTMT.
    Function(GMRef<GMFunction>),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum GMDataType {
    /// 64-bit floating point number.
    /// - Size on VM Stack: 8 bytes.
    Double = 0,

    // /// Does not really exist for some reason?
    // Float = 1,
    //
    /// 32-bit signed integer.
    /// - Size on VM Stack: 4 bytes.
    Int32 = 2,

    /// 64-bit signed integer.
    /// - Size on VM Stack: 8 bytes.
    Int64 = 3,

    /// Boolean, represented as 1 or 0, with a 32-bit integer.
    /// - Size on VM Stack: 4 bytes (for some reason).
    Boolean = 4,

    /// Dynamic type representing any GML value.
    /// Externally known as a structure called `RValue`.
    /// - Size on VM Stack: 16 bytes.
    Variable = 5,

    /// String, represented as a 32-bit ID.
    /// - Size on VM Stack: 4 bytes.
    String = 6,

    /// 16-bit signed integer.
    /// - Size on VM Stack: 4 bytes.
    /// > **Note**: `Int16` is not a valid data type on the VM Stack.
    /// It is immediately converted to `Int32` when pushing and is thus 4 bytes wide.
    Int16 = 15,
}

impl GMDataType {
    /// The size of a value of this data type on the VM Stack, in bytes.
    pub const fn size(self) -> u8 {
        match self {
            GMDataType::Int16 => 4,
            GMDataType::Int32 => 4,
            GMDataType::Int64 => 8,
            GMDataType::Double => 8,
            GMDataType::Boolean => 4,
            GMDataType::String => 4,
            GMDataType::Variable => 16,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GMInstanceType {
    Undefined,

    /// Represents the current `self` instance.
    Self_(Option<GMRef<GMGameObject>>),

    /// Instance ID in the Room -100000; used when the Variable Type is [`GMVariableType::Instance`].
    /// This doesn't exist in UTMT.
    RoomInstance(i16),

    /// Represents the `other` context, which has multiple definitions based on the location used.
    Other,

    /// Represents all active object instances.
    /// Assignment operations can perform a loop.
    All,

    /// Represents no object/instance.
    None,

    /// Used for global variables.
    Global,

    /// Used for GML built-in variables.
    Builtin,

    /// Used for local variables; local to their code script.
    Local,

    /// Instance is stored in a Variable data type on the top of the stack.
    StackTop,

    /// Used for function argument variables in GMS 2.3+.
    Argument,

    /// Used for static variables.
    Static,
}

impl Display for GMInstanceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Undefined => write!(f, "Undefined"),
            Self::Self_(None) => write!(f, "Self"),
            Self::Self_(Some(reference)) => write!(f, "Self<{}>", reference.index),
            Self::RoomInstance(instance_id) => write!(f, "RoomInstanceID<{instance_id}>"),
            Self::Other => write!(f, "Other"),
            Self::All => write!(f, "All"),
            Self::None => write!(f, "None"),
            Self::Global => write!(f, "Global"),
            Self::Builtin => write!(f, "Builtin"),
            Self::Local => write!(f, "Local"),
            Self::StackTop => write!(f, "StackTop"),
            Self::Argument => write!(f, "Argument"),
            Self::Static => write!(f, "Static"),
        }
    }
}

impl GMInstanceType {
    /// Convert an instance type to the "VARI version".
    /// In other words, convert the instance type to what
    /// it would be if it was in the 'VARI' chunk (`GMVariable.instance_type`)
    /// instead of in an instruction (`CodeVariable.instance_type`).
    #[must_use]
    pub(crate) fn as_vari(&self) -> Self {
        match self {
            Self::StackTop => Self::Self_(None),
            Self::Builtin => Self::Self_(None),
            Self::Self_(Some(_)) => Self::Self_(None),
            Self::RoomInstance(_) => Self::Self_(None),
            Self::Argument => Self::Builtin,
            Self::Other => Self::Self_(None),
            _ => self.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum GMVariableType {
    /// Used for normal single-dimension array variables.
    Array = 0x00,

    /// Used when referencing a variable on another variable, e.g. a chain reference.
    StackTop = 0x80,

    /// Normal variable access.
    Normal = 0xA0,

    /// Used when referencing variables on room instance IDs, e.g. something like `inst_01ABCDEF.x` in GML.
    Instance = 0xE0,

    /// GMS2.3+, multidimensional array with pushaf.
    ArrayPushAF = 0x10,

    /// GMS2.3+, multidimensional array with pushaf or popaf.
    ArrayPopAF = 0x90,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum GMComparisonType {
    /// "Less than" | `<`
    LessThan = 1,

    /// "Less than or equal to" | `<=`
    LessOrEqual = 2,

    /// "Equal to" | `==`
    Equal = 3,

    /// "Not equal to" | `!=`
    NotEqual = 4,

    /// "Greater than or equal to" | `>=`
    GreaterOrEqual = 5,

    /// "Greater than" | `>`
    GreaterThan = 6,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodeVariable {
    pub variable: GMRef<GMVariable>,
    pub variable_type: GMVariableType,
    pub instance_type: GMInstanceType,

    /// TODO: when does this happen?
    pub is_int32: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GMCodeValue {
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Double(f64),
    Boolean(bool),
    String(String),
    Variable(CodeVariable),
    /// Does not exist in UTMT. Added in order to support inline/anonymous functions.
    Function(GMRef<GMFunction>),
}

impl GMCodeValue {
    pub const fn data_type(&self) -> GMDataType {
        match self {
            Self::Int16(_) => GMDataType::Int16,
            Self::Int32(_) => GMDataType::Int32,
            Self::Function(_) => GMDataType::Int32, // Functions are not a "real" gm type; they're always int32
            Self::Variable(var) if var.is_int32 => GMDataType::Int32, // no idea when this happens
            Self::Int64(_) => GMDataType::Int64,
            Self::Double(_) => GMDataType::Double,
            Self::Boolean(_) => GMDataType::Boolean,
            Self::String(_) => GMDataType::String,
            Self::Variable(_) => GMDataType::Variable,
        }
    }
}
