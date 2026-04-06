//! Deltarune: Chapter 3 & 4
//! Chapters 1, 2, 4 (3 needs extra handling)
//! 2025-06-05 to now [2026-01-04]

use crate::gml::GMCode;
use crate::gml::assembly::assemble_instructions;
use crate::gml::insert_instructions;
use crate::gml::instruction::InstanceType;
use crate::gml::instruction::Instruction;
use crate::gml::instruction::VariableType;
use crate::prelude::*;
use crate::wad::elements::variable::GMVariable;

pub fn toggle(data: &mut GMData, enable: bool) -> Result<()> {
    let code_ref: GMRef<GMCode> = data
        .codes
        .ref_by_name("gml_Object_obj_initializer2_Create_0")?;
    super::replace_debug(data, code_ref, enable, InstanceType::Global)?;

    if !enable || data.codes.by_name("gml_Script_scr_flag_name_get").is_err() {
        return Ok(());
    }

    // need to modify flag retrieval code (currently only used in chapter 4)
    let code_ref = data.codes.ref_by_name("gml_GlobalScript_scr_flag_get")?;
    let code = data.codes.by_ref(code_ref)?;

    let index: u32 = find_insertion_point(data, code)?;

    let assembly: &str = r#"
        push.s "flagname"
        conv.s.v
        call variable_global_exists(argc=1)
        conv.v.b
        not.b
        jf 5
        push.s ""
        conv.s.v
        ret
    "#;
    let insertion: Vec<Instruction> = assemble_instructions(assembly, data)?;

    let code = data.codes.by_ref_mut(code_ref)?;
    insert_instructions(&mut code.instructions, index, &insertion)?;
    bail!(
        "TODO: debug activation for deltarune is currently broken (need to fix child gml_Script \
         execution offsets)"
    );
}

fn find_insertion_point(data: &GMData, code: &GMCode) -> Result<u32> {
    for i in 0..code.instructions.len() {
        let instr = &code.instructions[i];
        let Some(code_variable) = instr.variable() else {
            continue;
        };
        let variable: &GMVariable = data.variables.by_ref(code_variable.variable)?;
        if variable.name != "flagname" {
            continue;
        }
        // (assumes instancetype global)

        let ref_type = code_variable.variable_type;
        if ref_type != VariableType::Array {
            bail!("Expected array access for global.flagname, got {ref_type:?}");
        }

        // Found `push.v [array]global.flagname`

        let pred = |i: &Instruction| matches!(i, Instruction::BranchUnless { .. });
        let branch_index = code.instructions[..i].iter().rposition(pred);
        let idx = branch_index.ok_or("No branch instruction before global.flagname push")?;
        if i - idx > 8 {
            // branch is too far away, code is probably malformed
            bail!("Could not find a branch near (before) global.flagname push");
        }

        // nice, found the insertion point
        return Ok(idx as u32 + 1);
    }

    Err(err!("Could not find global.flagname push"))
}

// vanilla assembly looks like this (snippet):
//
// pushglb.v global.is_console
// conv.v.b
// not.b
// bf [10]                       // BRANCH INSTRUCTION
//
// >>>> should insert instructions here <<<<
//
// :[5]
// pushi.e -1
// push.v arg.argument0
// conv.v.i
// push.v [array]self.flagname   // PUSH INTO FLAGNAME
// pop.v.v local.v
// pushloc.v local.v
// call.i is_undefined(argc=1)
// conv.v.b
// bf [7]
//
// :[6]
// push.s "*unknown flag*"@50
// conv.s.v
// b [8]
//
// :[7]
// pushloc.v local.v
//
// :[8]
// ret.v
//
