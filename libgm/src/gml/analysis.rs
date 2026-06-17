// SPDX-License-Identifier: GPL-3.0-only
use crate::gml::Instruction;
use crate::gml::instruction::DataType;
use crate::prelude::*;

/// Information about the bytecode of this game
/// that needs work to extract.
///
/// You should therefore only call this function once per game
/// and keep the struct.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct CodeAnalysis {
    /// Whether Copy on Write (CoW) functionality is enabled for arrays.
    ///
    /// Between GameMaker 2.3 and 2022.2 this is guaranteed to be true.
    /// Afterward, it is set to false by default in the GameMaker IDE, but can
    /// be changed by the game developer.
    ///
    /// If there is a `setowner` instruction, then Array CoW must be enabled.
    ///
    /// By default, this is disabled.
    pub array_cow: bool,

    /// Whether this game was compiled with short-circuiting logic.
    ///
    /// This means that boolean `AND` and `OR` operations are guaranteed
    /// to stop executing when evaluating more would be useless:
    /// If the left hand side expression evaluated to false in an `AND`, the
    /// right  hand  side expression is not useful to evaluate, since `false
    /// and XXXXX` is always `false`. The same thing applies to when the LHS
    /// expression is `true` for `OR`: `true or XXXXX` is always `true`.
    ///
    /// Guaranteeing this logic is useful, since expressions
    /// can consist of function calls, which can mutate state.
    ///
    /// If there are `and`/`or` instructions with
    /// both data types boolean, then short-circuiting must be
    /// disabled, because it it would have branched away otherwise.
    ///
    /// By default, this feature is enabled.
    pub short_circuit: bool,
}

impl Default for CodeAnalysis {
    fn default() -> Self {
        Self { array_cow: false, short_circuit: true }
    }
}

/// Analyzes some information about the bytecode used in this game.
///
/// This has to loop over every instruction in every code entry,
/// so it's a good idea to only call this once per game.
///
/// None of these analyzed properties change when the data file is modified,
/// since this information is about how this game was compiled.
#[must_use]
pub fn analyze(data: &GMData) -> CodeAnalysis {
    let mut analysis = CodeAnalysis::default();

    for code in data.codes.elements() {
        for instruction in &code.instructions {
            match instruction {
                Instruction::And { lhs: DataType::Bool, rhs: DataType::Bool }
                | Instruction::Or { lhs: DataType::Bool, rhs: DataType::Bool } => {
                    analysis.short_circuit = false;
                }
                Instruction::SetArrayOwner => {
                    analysis.array_cow = true;
                }
                _ => {}
            }
        }
    }

    analysis
}
