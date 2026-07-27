/// Instructions that pop two values from the stack,
/// do something with them, and push the result.
///
/// The left-hand-side (LHS) type represents the value that is pushed first (and therefore popped second).
/// It is the second type encoded in the instruction (`type2`).
///
/// The right-hand-side (RHS) type represents the value that is pushed second (and therefore popped first).
/// It is the first type encoded in the instruction (`type1`).
///
/// The resulting value will have the first type encoded in the instruction (`type1`).
///
/// Most of these throw runtime errors when used with strings, except for `Add`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    /// Adds two numbers or concatenates two strings.
    Add,

    /// Subtracts two numbers.
    Subtract,

    /// Multiplies two numbers.
    Multiply,

    /// Divides two numbers.
    Divide,

    /// Performs a GML `div` operation (division with remainder).
    /// The second popped number (the dividend) is divided (with remainder)
    /// by the first popped number (the divisor).
    ///
    /// This operation is similar to [`Self::Modulus`], except it behaves
    /// differently for negative numbers.
    /// For example: `-19 rem 12 == -7` (not 5).
    Remainder,

    /// Performs a GML `mod` operation (`%`).
    /// The second popped number (the dividend) is modulo'd against the first popped number (the divisor).
    ///
    /// This operation is similar to [`Self::Remainder`], except it
    /// behaves differently for negative numbers. This `modulus` operation performs
    /// [Euclidean division](https://en.wikipedia.org/wiki/Euclidean_division).
    /// For example: `-19 rem 12 == 5` (not -7).
    Modulus,

    /// Performs bitwise or logical AND, depending on the data types.
    And,

    /// Performs bitwise or logical OR, depending on the data types.
    Or,

    /// Performs bitwise or logical XOR, depending on the data types.
    Xor,

    /// Performs a bitwise **left shift** operation (`<<`).
    /// The second popped number is shifted to the left by
    /// the first popped number (shift amount bits).
    ShiftLeft,

    /// Performs a bitwise **right shift** operation (`>>`).
    /// The second popped number is shifted to the right
    /// by the first popped number (shift amount bits).
    ShiftRight,
}
