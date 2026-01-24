//! Deltarune: Chapter 3 & 4
//! Chapter 3:
//! 2025-06-05 to now [2026-01-04]

use crate::{
    gamemaker::elements::variable::GMVariable,
    gml::{
        GMCode,
        assembly::assemble_instructions,
        instruction::{InstanceType, Instruction},
    },
    prelude::*,
};

// === Instruction Layout - Before ===
// pushim 0
// pop.v.i global.debug
//
// === Instruction Layout - After ===
// pushim 1
// pop.v.i global.debug
// pushim 0
// pop.v.i global.chemg_show_room
// pushim 0
// pop.v.i global.chemg_show_val

pub fn toggle(data: &mut GMData, enable: bool) -> Result<()> {
    let code_ref = data
        .codes
        .ref_by_name("gml_Object_obj_initializer2_Create_0")?;
    let (push_instr_index, is_enabled) = super::find_debug(data, code_ref, InstanceType::Global)?;

    if enable == is_enabled {
        // Debug mode already in correct state
        return Ok(());
    }

    // Enable/disable debug mode.
    let code: &mut GMCode = data.codes.by_ref_mut(code_ref)?;
    let integer = i16::from(enable);
    code.instructions[push_instr_index] = Instruction::PushImmediate { integer };

    // If enabling, add some more global variable declarations
    if !enable {
        return Ok(());
    }

    if let Some(Instruction::Pop { variable, .. }) = code.instructions.get(push_instr_index + 3) {
        let gm_variable: &GMVariable = data.variables.by_ref(variable.variable)?;
        if gm_variable.name == "chemg_show_room" {
            // Debug had been enabled here before.
            // Return instead of bloating the code with duplicate assignments.
            return Ok(());
        }
    }

    let assembly = "
         pushim 0
         pop.v.i global.chemg_show_room
         pushim 0
         pop.v.i global.chemg_show_val
    ";

    let instructions = assemble_instructions(assembly, data)?;
    let code: &mut GMCode = data.codes.by_ref_mut(code_ref)?;
    let idx = push_instr_index + 2;
    code.instructions.splice(idx..idx, instructions);

    Ok(())
}
