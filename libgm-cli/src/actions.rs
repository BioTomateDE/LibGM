use clap::ValueEnum;
use libgm::{
    gamemaker::{
        data::GMData,
        elements::{GMNamedListChunk, script::GMScript},
        reference::GMRef,
    },
    gml::{assembly::assemble_code, instructions::GMCode},
    prelude::*,
};

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    EnableDebug,
}

impl Action {
    pub fn perform(self, data: &mut GMData) -> Result<()> {
        match self {
            Self::EnableDebug => enable_debug(data).context("enabling debug mode"),
        }
    }
}

fn enable_debug(data: &mut GMData) -> Result<()> {
    log::info!("Enabling Debug");

    if data.general_info.game_name != "UNDERTALE" {
        return Err("Only Undertale is supported as of now".into());
    }

    let assembly = r#"
        pushim 1
        pop.v.v global.debug
    "#;
    let instructions = assemble_code(assembly, data)?;

    let script: &GMScript = data.scripts.by_name("SCR_GAMESTART")?;
    let code: GMRef<GMCode> = script.code.ok_or("Script does not have a code entry set")?;
    let code: &mut GMCode = data.codes.by_ref_mut(code)?;
    code.instructions.extend(instructions);

    Ok(())
}
