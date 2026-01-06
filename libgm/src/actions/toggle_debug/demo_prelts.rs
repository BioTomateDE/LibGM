//! Deltarune Demo: 1&2 demo before LTS
//! Chapter: 1, 2
//! 2021-09-17 to 2025-06-05

use crate::{
    gml::{
        assembly::assemble_instruction,
        instruction::{InstanceType, Instruction},
    },
    prelude::*,
};

pub fn toggle(data: &mut GMData, enable: bool) -> Result<()> {
    // Modify SCR_GAMESTART
    let script = data.scripts.by_name("SCR_GAMESTART")?;
    let code_ref = script.code.ok_or("Script does not reference code entry")?;
    super::replace_debug(data, code_ref, enable, InstanceType::Global)?;

    // Modify obj_debugcontroller Creation
    let code_ref = data
        .codes
        .ref_by_name("gml_Object_obj_debugcontroller_ch1_Create_0")?;
    super::replace_debug(data, code_ref, enable, InstanceType::Self_)?;

    // Modify obj_debugProfiler Creation
    let instructions = if enable {
        let pushim = Instruction::PushImmediate { integer: 0 };
        let pop = assemble_instruction("pop.v.i self.cutsceneshow", data)?;
        vec![pushim, pop]
    } else {
        vec![]
    };
    let code = data
        .codes
        .by_name_mut("gml_Object_obj_debugProfiler_Create_0")?;
    code.instructions = instructions;
    // I am currently not checking the instruction count, unlike the original "UTDR Scripts/Debug.csx".
    // If there are issues, lmk

    Ok(())
}
