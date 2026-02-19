use macros::num_enum;

/// How to compare values.
/// Used in the [`Comparison`] instruction (`cmp`).
///
/// [`Comparison`]: crate::gml::Instruction::Compare
#[num_enum(u8)]
pub enum ComparisonType {
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
