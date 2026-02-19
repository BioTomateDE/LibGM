use crate::{
    gml::{Instruction, instruction::DataType},
    prelude::*,
};

/// Information about the bytecode of this game
/// that needs work to extract.
///
/// You should therefore only call this function once per game
/// and keep the struct.
#[expect(clippy::manual_non_exhaustive)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CodeAnalysis {
    /// Whether Copy on Write (Cow) functionality is enabled for arrays.
    ///
    /// Between GameMaker 2.3 and 2022.2 this is guaranteed to be `true`.
    /// Afterwards, it is set to false by default in the GameMaker IDE, but can be changed by a game's developer.
    ///
    /// This is detected by the usage of [`Instruction::SetArrayOwner`].
    ///
    /// By default, this is disabled.
    pub uses_array_copy_on_write: bool,

    /// Whether this game was compiled with short-circuiting logic.
    ///
    /// This means that boolean `AND` and `OR` operations are guaranteed
    /// to stop executing when evaluating more would be useless:
    /// If the left hand side expression evaluted to false in an `AND`, the right  hand  side
    /// expression is not useful to evaluate, since `false and XXXXX` is always `false`.
    /// The same thing applies to when the LHS expression is `true` for `OR`: `true or XXXXX` is always `true`.
    ///
    /// Guaranteeing this logic is useful, since expressions
    /// can consist of function calls, which can mutate state.
    ///
    /// This is detected by the inexistence of `and.b.b` and `or.b.b` instructions.
    ///
    /// By default, this is enabled.
    pub uses_short_circuit: bool,

    // Make this struct impossible to construct.
    // This doubles as a #[non_exhaustive]
    _private: (),
}

impl Default for CodeAnalysis {
    fn default() -> Self {
        Self {
            uses_array_copy_on_write: false,
            uses_short_circuit: true,
            _private: (),
        }
    }
}

/// Analyzes some information about the bytecode used in this game.
///
/// This has to loop over every instruction in every code entry,
/// so it's a good idea to only call this once per game.
///
/// None of these analyzed properties change when the data file is modified,
/// since this information is about how this game was compiled.
pub fn analyze(data: &GMData) -> CodeAnalysis {
    let mut analysis = CodeAnalysis::default();

    for code in &data.codes {
        for instruction in &code.instructions {
            match instruction {
                Instruction::And {
                    lhs: DataType::Boolean,
                    rhs: DataType::Boolean,
                }
                | Instruction::Or {
                    lhs: DataType::Boolean,
                    rhs: DataType::Boolean,
                } => {
                    analysis.uses_short_circuit = false;
                },
                Instruction::SetArrayOwner => {
                    analysis.uses_array_copy_on_write = true;
                },
                _ => {},
            }
        }
    }

    analysis
}

impl GMData {
    /// Analyzes some information about the bytecode used in this game.
    ///
    /// For more information, see the [`analyze`] function in [`crate::gml::analysis`].
    pub fn analyze_code(&self) -> CodeAnalysis {
        analyze(self)
    }
}
