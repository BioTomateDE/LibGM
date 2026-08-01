// SPDX-License-Identifier: GPL-3.0-only
//! Deltarune Demo: 1&2 demo before LTS
//! Chapter: 1, 2
//! 2021-09-17 to 2025-06-05

use crate::actions::toggle_debug::create_event;
use crate::gml::assembly::assemble_instruction;
use crate::gml::instruction::InstanceType;
use crate::gml::instruction::Instruction;
use crate::prelude::*;

pub fn toggle(data: &mut GMData, enable: bool) -> Result<()> {
    log::debug!("Detected Deltarune PreLTS Demo Chapters 1&2");
    // Modify scr_gamestart
    let script = data.scripts.by_name("scr_gamestart", &data.strings)?;
    super::replace_debug(data, script.code, enable, InstanceType::Global)?;

    // Modify obj_debugcontroller Creation
    let obj = data.game_objects.by_name("obj_debugcontroller_ch1", &data.strings)?;

    super::replace_debug(data, create_event(obj)?, enable, InstanceType::Self_)?;

    // Modify obj_debugProfiler Creation
    let instructions = if enable {
        let pushim = Instruction::PushImmediate { integer: 0 };
        let pop = assemble_instruction("pop.v.i self.cutsceneshow", data)?;
        vec![pushim, pop]
    } else {
        vec![]
    };
    let obj = data.game_objects.by_name("obj_debugProfiler", &data.strings)?;
    let code = data.codes.by_ref_mut(create_event(obj)?)?;
    code.instructions = instructions;

    // I am currently not checking the instruction count, unlike the original "UTDR
    // Scripts/Debug.csx". If there are issues, lmk

    Ok(())
}
