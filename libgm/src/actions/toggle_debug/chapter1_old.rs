// SPDX-License-Identifier: GPL-3.0-only
//! Deltarune Demo Chapter 1: before 1&2 demo
//! Chapter: 1
//! 2018-08-31 to 2021-09-17

use crate::gml::assembly::assemble_instruction;
use crate::gml::instruction::Instruction;
use crate::prelude::*;

pub fn toggle(data: &mut GMData, enable: bool) -> Result<()> {
    log::debug!("Detected old Deltarune Chapter 1");
    let pushim = Instruction::PushImmediate { integer: i16::from(enable) };
    let pop = assemble_instruction("pop global.debug", data)?;
    let push = assemble_instruction("push.v global.debug", data)?;
    let ret = Instruction::Return;
    let instructions = vec![pushim, pop, push, ret];

    let code = data
        .codes
        .by_name_mut("gml_Script_scr_debug", &data.strings)?;
    code.instructions = instructions;
    Ok(())
}
