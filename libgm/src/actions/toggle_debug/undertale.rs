//! Any Undertale or NXTALE version
//! 2015-09-15 to eternity

use crate::{
    gamemaker::elements::script::GMScript,
    gml::instruction::{GMCode, InstanceType},
    prelude::*,
};

pub fn toggle(data: &mut GMData, enable: bool) -> Result<()> {
    let script: &GMScript = data.scripts.by_name("SCR_GAMESTART")?;
    let code_ref: GMRef<GMCode> = script.code.ok_or("Script does not have a code entry set")?;
    super::replace_debug(data, code_ref, enable, InstanceType::Global)
}
