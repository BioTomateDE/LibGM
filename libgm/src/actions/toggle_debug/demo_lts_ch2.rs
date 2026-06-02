// SPDX-License-Identifier: GPL-3.0-only
//! Deltarune Demo: LTS, Chapter 2
//! Chapter: 2
//! 2025-06-05 to now [2026-01-04]

use crate::gml::assembly::assemble_instructions;
use crate::gml::instruction::Instruction;
use crate::prelude::*;

pub fn toggle(data: &mut GMData, enable: bool) -> Result<()> {
    log::debug!("Detected Deltarune LTS Demo Chapter 2");
    // Placeholder in line 2
    let assembly = "
        br 5
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

    let code = data
        .codes
        .by_name_mut("gml_Script_scr_debug", &data.strings)?;
    code.instructions = instructions;
    Ok(())
}
