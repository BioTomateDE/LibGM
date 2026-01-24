//! Deltarune: Chapter 3 & 4
//! Chapters 1, 2, 4 (3 needs extra handling)
//! 2025-06-05 to now [2026-01-04]

use crate::{
    gamemaker::elements::variable::GMVariable,
    gml::{
        GMCode,
        instruction::{CodeVariable, InstanceType, Instruction, PushValue, VariableType},
    },
    prelude::*,
};

#[expect(unused)]
pub fn toggle(data: &mut GMData, enable: bool) -> Result<()> {
    let code_ref: GMRef<GMCode> = data
        .codes
        .ref_by_name("gml_Object_obj_initializer2_Create_0")?;
    super::replace_debug(data, code_ref, enable, InstanceType::Global)?;

    if data.codes.by_name("gml_Script_scr_flag_name_get").is_err() {
        return Ok(());
    }

    let code_ref = data.codes.ref_by_name("gml_GlobalScript_scr_flag_get")?;
    let code = data.codes.by_ref(code_ref)?;

    log::warn!("This is untested, undefined behavior could occur!");
    bail!("modifying control flow is hard :c");
    // TODO: implement

    for i in 0..code.instructions.len() {
        let instr = &code.instructions[i];
        let code_variable: &CodeVariable = match instr {
            Instruction::PushGlobal { variable }
            | Instruction::Push { value: PushValue::Variable(variable) } => variable,
            _ => continue,
        };

        let variable: &GMVariable = data.variables.by_ref(code_variable.variable)?;
        if variable.name != "flagname" {
            continue;
        }

        let instance_type = variable
            .modern_data
            .as_ref()
            .ok_or("WAD 15+ data not set in Deltarune (presumably Chapter 4)")?
            .instance_type;

        if instance_type != InstanceType::Global {
            continue;
        }

        let ref_type = code_variable.variable_type;
        if ref_type != VariableType::Array {
            bail!("Expected array access for global.flagname, got {ref_type:?}");
        }

        // Found `push.v [array]global.flagname`
    }

    Ok(())
}
