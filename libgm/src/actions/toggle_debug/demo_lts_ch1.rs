// SPDX-License-Identifier: GPL-3.0-only
//! Deltarune Demo: LTS, Chapter 1
//! Chapter: 1
//! 2025-06-05 to now [2026-01-04]

use crate::gml::Code;
use crate::gml::instruction::InstanceType;
use crate::prelude::*;

pub fn toggle(data: &mut GMData, enable: bool) -> Result<()> {
    log::debug!("Detected Deltarune LTS Demo Chapter 1");
    let code_ref: GMRef<Code> = data
        .codes
        .ref_by_name("gml_Object_obj_debugcontroller_Create_0", &data.strings)?;
    super::replace_debug(data, code_ref, enable, InstanceType::Global)
}
