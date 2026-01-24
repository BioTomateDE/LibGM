use macros::num_enum;

/// A primitive data type used in instructions.
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
    /// The size of a value of this data type on the VM Stack, in bytes.
    #[must_use]
    pub const fn size(self) -> u8 {
        match self {
            Self::Int16 | Self::Int32 | Self::Boolean | Self::String => 4,
            Self::Int64 | Self::Double => 8,
            Self::Variable => 16,
        }
    }
}
