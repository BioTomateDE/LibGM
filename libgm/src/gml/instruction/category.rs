use crate::gml::Instruction;

/// An (arbitrary) category of instruction types.
///
/// This classification does not exist in GameMaker,
/// I just made it for organization.
///
/// NOTE: This may be changed, more categories might get added.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    /// This category includes the following instructions:
    /// * [`Instruction::Negate`]
    /// * [`Instruction::Add`]
    /// * [`Instruction::Subtract`]
    /// * [`Instruction::Multiply`]
    /// * [`Instruction::Divide`]
    /// * [`Instruction::Remainder`]
    /// * [`Instruction::Modulus`]
    Arithmetic,

    /// This category includes the following instructions:
    /// * [`Instruction::Not`]
    /// * [`Instruction::And`]
    /// * [`Instruction::Or`]
    /// * [`Instruction::Xor`]
    /// * [`Instruction::ShiftLeft`]
    /// * [`Instruction::ShiftRight`]
    Bitwise,

    /// This category includes the following instructions:
    /// * [`Instruction::Exit`]
    /// * [`Instruction::Return`]
    /// * [`Instruction::Branch`]
    /// * [`Instruction::BranchIf`]
    /// * [`Instruction::BranchUnless`]
    /// * [`Instruction::PushWithContext`]
    /// * [`Instruction::PopWithContext`]
    /// * [`Instruction::PopWithContextExit`]
    ControlFlow,

    /// This category includes the following instructions:
    /// * [`Instruction::Push`]
    /// * [`Instruction::PushLocal`]
    /// * [`Instruction::PushGlobal`]
    /// * [`Instruction::PushBuiltin`]
    /// * [`Instruction::PushImmediate`]
    /// * [`Instruction::PushReference`]
    Push,

    /// This category contains all instructions that do not have one of the
    /// categories above assigned (yet).
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

    /// Whether this instruction belongs to the category [`Category::Arithmetic`].
    ///
    /// This includes arithmetic operations such as `Add` or `Divide`,
    /// but not bitwise operations such as `And` or `Xor`.
    #[must_use]
    pub const fn is_arithmetic(&self) -> bool {
        matches!(self.category(), Category::Arithmetic)
    }

    /// Whether this instruction is [`Category::Bitwise`].
    #[must_use]
    pub const fn is_bitwise(&self) -> bool {
        matches!(self.category(), Category::Bitwise)
    }

    /// Whether this instruction is [`Category::ControlFlow`].
    #[must_use]
    pub const fn is_control_flow(&self) -> bool {
        matches!(self.category(), Category::ControlFlow)
    }

    /// Whether this instruction is [`Category::Push`].
    #[must_use]
    pub const fn is_push(&self) -> bool {
        matches!(self.category(), Category::Push)
    }
}
