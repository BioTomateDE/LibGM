//! Deltarune Demo: LTS, Chapter 2
//! Chapter: 2
//! 2025-06-05 to now [2026-01-04]

use crate::{
    gamemaker::elements::script::GMScript,
    gml::{
        assembly::assemble_instructions,
        instruction::{GMCode, Instruction},
    },
    prelude::*,
};

pub fn toggle(data: &mut GMData, enable: bool) -> Result<()> {
    // Placeholder in line 2
    let assembly = "
        jmp 5
        pushim 1337
        conv.i.v
        ret
        exit
        push.i (function)gml_Script_scr_debug
        conv.i.v
        pushim -1
        conv.i.v
        call method(argc=2)
        dup.v 0
        pushim -1
        pop.v.v [stacktop]self.scr_debug
        popz.v
    ";

    let pushim = Instruction::PushImmediate { integer: i16::from(enable) };
    let mut instructions = assemble_instructions(assembly, data)?;
    instructions[1] = pushim;

    let script: &GMScript = data.scripts.by_name("scr_debug")?;
    let code_ref: GMRef<GMCode> = script.code.ok_or("Script does not have a code entry set")?;
    let code = data.codes.by_ref_mut(code_ref)?;
    code.instructions = instructions;
    Ok(())
}
