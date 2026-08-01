// SPDX-License-Identifier: GPL-3.0-only
//! Any Undertale or NXTALE version
//! 2015-09-15 to eternity

use crate::gml::instruction::InstanceType;
use crate::prelude::*;

pub fn toggle(data: &mut GMData, enable: bool) -> Result<()> {
    log::debug!("Detected Undertale");
    let script = data.scripts.by_name("SCR_GAMESTART", &data.strings)?;
    super::replace_debug(data, script.code, enable, InstanceType::Global)
}
