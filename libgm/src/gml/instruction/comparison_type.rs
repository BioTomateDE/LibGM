// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;

/// How to compare values.
/// Used in the [`Comparison`] instruction (`cmp`).
///
/// [`Comparison`]: crate::gml::Instruction::Compare
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComparisonType {
    /// "Less than" | `<`
    LessThan,

    /// "Less than or equal to" | `<=`
    LessOrEqual,

    /// "Equal to" | `==`
    Equal,

    /// "Not equal to" | `!=`
    NotEqual,

    /// "Greater than or equal to" | `>=`
    GreaterOrEqual,

    /// "Greater than" | `>`
    GreaterThan,
}

impl ComparisonType {
    pub fn from_u8(raw: u8) -> Result<Self> {
        Ok(match raw {
            1 => Self::LessThan,
            2 => Self::LessOrEqual,
            3 => Self::Equal,
            4 => Self::NotEqual,
            5 => Self::GreaterOrEqual,
            6 => Self::GreaterThan,
            _ => bail!("Invalid Comparison Type {raw} ({raw:04X})"),
        })
    }

    #[must_use]
    pub fn as_u8(self) -> u8 {
        match self {
            Self::LessThan => 1,
            Self::LessOrEqual => 2,
            Self::Equal => 3,
            Self::NotEqual => 4,
            Self::GreaterOrEqual => 5,
            Self::GreaterThan => 6,
        }
    }
}
