// SPDX-License-Identifier: GPL-3.0-only
use crate::gml::assembly::assemble_instructions;
use crate::prelude::*;

impl GMData {
    /// Adds code to save the entire game state with S and load it immediately with L.
    ///
    /// This function currently only supports Undertale.
    pub fn enable_savestates(&mut self) -> Result<()> {
        enable_savestates(self)
    }
}

fn enable_savestates(data: &mut GMData) -> Result<()> {
    let name = &data.general_info.game_name;
    if !(name.contains("UNDERTALE") || name == "NXTALE") {
        bail!("Savestates currently only work for Undertale");
    }

    // TODO: expand functionality
    data.functions.make("game_load");
    data.functions.make("game_save");
    data.functions.make("keyboard_check_pressed");
    data.functions.make("audio_stop_all");
    // ord("L") = 76
    // ord("S") = 83
    let asm = r#"
        pushim 76
        conv.i.v
        call keyboard_check_pressed(argc=1)
        conv.v.b
        jf 10
        push.s "savestate"
        conv.s.v
        call audio_stop_all(argc=0)
        popz.v
        call game_load(argc=1)
        popz.v

        pushim 83
        conv.i.v
        call keyboard_check_pressed(argc=1)
        conv.v.b
        jf 7
        push.s "savestate"
        conv.s.v
        call game_save(argc=1)
        popz.v
    "#;
    let instrs = assemble_instructions(asm, data)?;

    let obj_time = data.game_objects.by_name_mut("obj_time")?;
    obj_time.visible = true;

    let code = data.codes.by_name_mut("gml_Object_obj_time_Draw_64")?;
    code.instructions = instrs;

    Ok(())
}
