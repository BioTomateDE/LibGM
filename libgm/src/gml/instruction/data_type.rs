use macros::num_enum;

/// A primitive data type used in instructions.
///
/// Note that this neither contains asset types (see [`AssetReference`])
/// nor structs/enums nor functions.
/// All of those are represented as [`DataType::Int32`].
///
/// The `variable` type is a bit weird, since it does not have
/// to actually be tied to a variable and is also not usable as a pointer/reference.
/// For example, you can (and will have to) convert pushed constant integers
/// into the [`DataType::Variable`] type using the [`Convert`] instruction,
/// even if that value is not even tied to an actual variable.
/// TODO(doc): Please conduct more research on this and improve these docs.
///
/// Another notable thing is that [`DataType::Int16`] is only valid for
/// pushing immediate values using [`PushImmediate`] (or [`Push`]).
/// Those instructions immediately convert the integer to an `Int32`.
/// Therefore, an `Int16` never actually exists on the stack and all
/// other instructions using this data type would be malformed.
///
/// Yet another notable thing is that there theoretically exists (or used to exist?)
/// a data type for single precision scalar floating point value (`Float`) with raw value 1.
/// However, it seems to be unused since YoYoGames prefers using `Double` instead.
///
/// [`AssetReference`]: crate::gml::instruction::AssetReference
/// [`Convert`]: crate::gml::Instruction::Convert
/// [`PushImmediate`]: crate::gml::Instruction::PushImmediate
/// [`Push`]: crate::gml::Instruction::Push
#[num_enum(u8)]
pub enum DataType {
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
    ///
    /// It is immediately converted to `Int32` when pushing and is thus 4 bytes wide.
    Int16 = 15,
}

impl DataType {
    /// The size of a value of this data type on the VM Stack, in multiples of 4 bytes.
    /// This is the unit used in `jump_offset` of branch instructions.
    #[must_use]
    pub const fn size4(self) -> u8 {
        match self {
            Self::Int16 | Self::Int32 | Self::Boolean | Self::String => 1,
            Self::Int64 | Self::Double => 2,
            Self::Variable => 4,
        }
    }

    /// The size of a value of this data type on the VM Stack, in bytes.
    #[must_use]
    pub const fn size(self) -> u8 {
        //TODO(break): change type to u32
        self.size4() * 4
    }
}
