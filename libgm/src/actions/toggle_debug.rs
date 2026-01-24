mod chapter1_old;
mod chapter3;
mod deltarune;
mod demo_lts_ch1;
mod demo_lts_ch2;
mod demo_prelts;
mod undertale;

use std::fmt::Display;

use crate::{
    gamemaker::elements::variable::GMVariable,
    gml::{
        GMCode,
        instruction::{InstanceType, Instruction, PushValue},
    },
    prelude::*,
};

impl GMData {
    /// Tries to enable or disable debug mode, based on the argument.
    ///
    /// This function currently supports:
    /// * Undertale
    /// * NXTALE
    /// * Deltarune Chapter 1 (aka. `SURVEY_PROGRAM`)
    /// * Deltarune Chapter 1&2 Demo (Old)
    /// * Deltarune Chapter 1&2 LTS Demo (New)
    /// * Deltarune Chapters 1-4 (paid game)
    pub fn toggle_debug(&mut self, enable: bool) -> Result<()> {
        toggle_debug(self, enable)
    }

    /// Enables debug mode.
    ///
    /// For more information, see [`Self::toggle_debug`].
    pub fn enable_debug(&mut self) -> Result<()> {
        self.toggle_debug(true)
    }

    /// Disables debug mode.
    ///
    /// For more information, see [`Self::toggle_debug`].
    pub fn disable_debug(&mut self) -> Result<()> {
        self.toggle_debug(false)
    }
}

fn toggle_debug(data: &mut GMData, enable: bool) -> Result<()> {
    let gen8 = &data.general_info;
    let display_name: &str = &gen8.display_name;
    let internal_name: &str = &gen8.game_name;

    if internal_name.contains("UNDERTALE") || internal_name == "NXTALE" {
        return undertale::toggle(data, enable);
    }

    match display_name {
        "SURVEY_PROGRAM" => return chapter1_old::toggle(data, enable),
        "DELTARUNE Chapter 1&2" => return demo_prelts::toggle(data, enable),
        "DELTARUNE Chapter 3" => return chapter3::toggle(data, enable),
        "DELTARUNE Chapter 4" => return deltarune::toggle(data, enable),
        _ => {},
    }

    if display_name == "DELTARUNE Chapter 1" {
        return if data.game_objects.by_name("obj_event_manager").is_ok() {
            deltarune::toggle(data, enable)
        } else if gen8.version.is_version_at_least((2, 3)) {
            demo_lts_ch1::toggle(data, enable)
        } else {
            chapter1_old::toggle(data, enable)
        };
    }

    if display_name == "DELTARUNE Chapter 2" {
        return if data.game_objects.by_name("obj_event_manager").is_ok() {
            deltarune::toggle(data, enable)
        } else {
            demo_lts_ch2::toggle(data, enable)
        };
    }

    bail!("Could not detect Undertale or Deltarune from game {display_name:?}");
}

fn find_debug(
    data: &GMData,
    code_ref: GMRef<GMCode>,
    instance_type: InstanceType,
) -> Result<(usize, bool)> {
    let code = data.codes.by_ref(code_ref)?;

    for i in 0..code.instructions.len() - 1 {
        let potential_push: &Instruction = &code.instructions[i];
        let potential_pop: &Instruction = &code.instructions[i + 1];

        let Instruction::Pop { variable, .. } = &potential_pop else {
            continue;
        };
        if variable.instance_type != instance_type {
            continue;
        }
        let gm_variable: &GMVariable = data.variables.by_ref(variable.variable)?;
        if gm_variable.name != "debug" {
            continue;
        }

        // Found a `pop` into `debug`. Now extract the push integer value.
        let is_enabled: bool = match *potential_push {
            Instruction::PushImmediate { integer }
            | Instruction::Push { value: PushValue::Int16(integer) } => int_to_bool(integer)?,
            Instruction::Push { value: PushValue::Int32(integer) } => int_to_bool(integer)?,
            Instruction::Push { value: PushValue::Boolean(bool) } => bool,
            // Do other datatypes have to be supported?
            _ => bail!(
                "Expected Instruction before Pop to be an integer push, found {potential_push:?}"
            ),
        };

        return Ok((i, is_enabled));
    }

    Err("Could not find any pop instruction for debug variable".into())
}

fn replace_debug(
    data: &mut GMData,
    code_ref: GMRef<GMCode>,
    enable: bool,
    instance_type: InstanceType,
) -> Result<()> {
    let (instruction_index, is_enabled) = find_debug(data, code_ref, instance_type)?;

    if enable == is_enabled {
        // Debug mode already in correct state
        return Ok(());
    }

    // Enable/disable debug mode.
    let code: &mut GMCode = data.codes.by_ref_mut(code_ref)?;
    let integer = i16::from(enable);
    code.instructions[instruction_index] = Instruction::PushImmediate { integer };
    Ok(())
}

fn int_to_bool<I: Display + Into<i32>>(integer: I) -> Result<bool> {
    let integer: i32 = integer.into();
    match integer {
        0 => Ok(false),
        1 => Ok(true),
        _ => bail!("Expected debug variable to be set to either 0 or 1, found {integer}"),
    }
}
