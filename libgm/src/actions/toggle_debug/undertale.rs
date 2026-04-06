//! Any Undertale or NXTALE version
//! 2015-09-15 to eternity

use crate::{gml::instruction::InstanceType, prelude::*};

pub fn toggle(data: &mut GMData, enable: bool) -> Result<()> {
    let code_ref = data.scripts.code_ref_by_name("SCR_GAMESTART")?;
    super::replace_debug(data, code_ref, enable, InstanceType::Global)
}
