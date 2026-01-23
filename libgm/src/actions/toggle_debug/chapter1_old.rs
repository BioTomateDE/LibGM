//! Deltarune Demo Chapter 1: before 1&2 demo
//! Chapter: 1
//! 2018-08-31 to 2021-09-17

use crate::{
    gml::{assembly::assemble_instruction, instruction::Instruction},
    prelude::*,
};

pub fn toggle(data: &mut GMData, enable: bool) -> Result<()> {
    let pushim = Instruction::PushImmediate { integer: i16::from(enable) };
    let pop = assemble_instruction("pop global.debug", data)?;
    let push = assemble_instruction("push.v global.debug", data)?;
    let ret = Instruction::Return;
    let instructions = vec![pushim, pop, push, ret];

    let script = data.scripts.by_name("scr_debug")?;
    let code_ref = script.code.ok_or("Script does not reference code entry")?;
    let code = data.codes.by_ref_mut(code_ref)?;
    code.instructions = instructions;
    Ok(())
}
