use libgm::{
    gamemaker::elements::{script::GMScript, variable::GMVariable},
    gml::instruction::{GMCode, InstanceType, Instruction, PushValue},
    prelude::*,
};

pub fn perform(data: &mut GMData) -> Result<()> {
    log::info!("Enabling Debug");

    assert_undertale(data)?;

    let script: &GMScript = data.scripts.by_name("SCR_GAMESTART")?;
    let code_ref: GMRef<GMCode> = script.code.ok_or("Script does not have a code entry set")?;
    let code: &GMCode = data.codes.by_ref(code_ref)?;

    for i in 0..code.instructions.len() - 1 {
        let potential_push: &Instruction = &code.instructions[i];
        let potential_pop: &Instruction = &code.instructions[i + 1];

        let Instruction::Pop { variable, .. } = &potential_pop else {
            continue;
        };
        if variable.instance_type != InstanceType::Global {
            continue;
        }
        let gm_variable: &GMVariable = data.variables.by_ref(variable.variable)?;
        if gm_variable.name != "debug" {
            continue;
        }

        // Found a `pop` into `global.debug`. Now extract the push integer value.
        let int: i32 = match *potential_push {
            Instruction::PushImmediate { integer } => i32::from(integer),
            Instruction::Push { value: PushValue::Int16(integer) } => i32::from(integer),
            Instruction::Push { value: PushValue::Int32(integer) } => integer,
            Instruction::Push { value: PushValue::Boolean(bool) } => i32::from(bool),
            // Do other datatypes have to be supported?
            _ => {
                return Err(format!(
                    "Expected Instruction before Pop to be an integer push, found {potential_push:?}"
                )
                .into());
            },
        };

        if int == 1 {
            // If debug was already enabled, just return.
            return Ok(());
        }
        if int != 0 {
            return Err(
                format!("Expected global.debug to be set to either 0 or 1, found {int}").into(),
            );
        }

        // Debug was disabled, enable it.
        let code: &mut GMCode = data.codes.by_ref_mut(code_ref)?;
        code.instructions[i] = Instruction::PushImmediate { integer: 1 };
        return Ok(());
    }

    Err("Could not find any pop instruction for `global.debug`".into())
}

fn assert_undertale(data: &GMData) -> Result<()> {
    let gen8 = &data.general_info;
    let names: [&str; 3] = [&gen8.display_name, &gen8.game_name, &gen8.game_file_name];
    if names.into_iter().any(contains_undertale) {
        Ok(())
    } else {
        Err(Error::from(format!(
            "Only Undertale is supported as of now (game name is {:?})",
            gen8.display_name
        )))
    }
}

#[must_use]
fn contains_undertale(string: &str) -> bool {
    string.to_ascii_lowercase().contains("undertale")
}
