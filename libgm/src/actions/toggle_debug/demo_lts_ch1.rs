//! Deltarune Demo: LTS, Chapter 1
//! Chapter: 1
//! 2025-06-05 to now [2026-01-04]

use crate::{
    gml::instruction::{GMCode, InstanceType},
    prelude::*,
};

pub fn toggle(data: &mut GMData, enable: bool) -> Result<()> {
    let code_ref: GMRef<GMCode> = data
        .codes
        .ref_by_name("gml_Object_obj_debugcontroller_Create_0")?;
    super::replace_debug(data, code_ref, enable, InstanceType::Global)
}
