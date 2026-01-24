use crate::gml::Instruction;

/// An (arbitrary) category of instruction types.
///
/// This classification does not exist in GameMaker,
/// I just made it for organization.
///
/// NOTE: This may be changed, more categories might get added.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Category {
    Arithmetic,
    Bitwise,
    ControlFlow,
    Push,
    Other,
}

impl Instruction {
    #[must_use]
    pub const fn category(&self) -> Category {
        match self {
            Self::Multiply { .. }
            | Self::Divide { .. }
            | Self::Remainder { .. }
            | Self::Modulus { .. }
            | Self::Add { .. }
            | Self::Subtract { .. }
            | Self::Negate { .. } => Category::Arithmetic,

            Self::And { .. }
            | Self::Or { .. }
            | Self::Xor { .. }
            | Self::Not { .. }
            | Self::ShiftLeft { .. }
            | Self::ShiftRight { .. } => Category::Bitwise,

            Self::Return
            | Self::Exit
            | Self::Branch { .. }
            | Self::BranchIf { .. }
            | Self::BranchUnless { .. }
            | Self::PushWithContext { .. }
            | Self::PopWithContext { .. }
            | Self::PopWithContextExit => Category::ControlFlow,

            Self::Push { .. }
            | Self::PushLocal { .. }
            | Self::PushGlobal { .. }
            | Self::PushBuiltin { .. }
            | Self::PushImmediate { .. }
            | Self::PushReference { .. } => Category::Push,

            _ => Category::Other,
        }
    }

    /// Whether this instruction is an arithmetic operation such as `Add` or `Divide`.
    ///
    /// This does not include bitwise operations such as `And`.
    #[must_use]
    pub const fn is_arithmetic(&self) -> bool {
        matches!(self.category(), Category::Arithmetic)
    }

    #[must_use]
    pub const fn is_bitwise(&self) -> bool {
        matches!(self.category(), Category::Bitwise)
    }

    #[must_use]
    pub const fn is_control_flow(&self) -> bool {
        matches!(self.category(), Category::ControlFlow)
    }

    #[must_use]
    pub const fn is_push(&self) -> bool {
        matches!(self.category(), Category::Push)
    }
}
