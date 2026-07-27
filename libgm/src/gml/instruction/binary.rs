/// Instructions that pop two values from the stack,
/// do something with them, and push the result.
///
/// Most of these throw runtime errors when used with strings, except for `Add`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    /// Adds two numbers or concatenates two strings.
    /// For string concatenatation, the second popped string is appended/concatenated to the the first popped string.
    Add,

    /// Subtracts the second popped number (the subtrahend) by the first popped number (the minuend).
    Subtract,

    /// Multiplies two numbers.
    Multiply,

    /// Divides the second popped number (the dividend) by the first popped number (the divisor).
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
