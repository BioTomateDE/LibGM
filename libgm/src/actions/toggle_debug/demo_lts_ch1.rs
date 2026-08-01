// SPDX-License-Identifier: GPL-3.0-only
//! Deltarune Demo: LTS, Chapter 1
//! Chapter: 1
//! 2025-06-05 to now [2026-01-04]

use crate::actions::toggle_debug::create_event;
use crate::gml::instruction::InstanceType;
use crate::prelude::*;

pub fn toggle(data: &mut GMData, enable: bool) -> Result<()> {
    log::debug!("Detected Deltarune LTS Demo Chapter 1");
    let obj = data.game_objects.by_name("obj_debugcontroller", &data.strings)?;
    super::replace_debug(data, create_event(obj)?, enable, InstanceType::Global)
}
