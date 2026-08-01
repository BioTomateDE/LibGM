// SPDX-License-Identifier: GPL-3.0-only
use crate::gml::Code;
use crate::gml::Instruction;
use crate::gml::ModernCodeData;
use crate::gml::instruction::CodeVariable;
use crate::gml::instruction::InstanceType;
use crate::gml::instruction::PushValue;
use crate::gml::instruction::VariableType;
use crate::prelude::*;
use crate::wad::GMVersion;
use crate::wad::elem::variable::Variable;

pub fn wad14_to_wad15(data: &mut GMData) -> Result<()> {
    if data.meta.version >= GMVersion::Wad15 {
        bail!("Data with version {} is already WAD15+", data.meta.version);
    }
    for idx in 0..data.codes.len() {
        let code = &data.codes.elems[idx];
        let arg_count = get_arg_count(code, data)? as u16;
        println!("{} has argc {arg_count}", code.name.display(&data.strings));
        let modern_data = ModernCodeData {
            local_count: 0, // eh
            arg_count,
            weird_local_flag: false,
            execution_offset: 0,
            parent: GMRef::none(),
        };
        // reborrow because of woke
        let code = &mut data.codes.elems[idx];
        code.modern_data = Some(modern_data);
    }

    // fix for undertale crashing immediately (other games unaffected)
    fix_undertale(data);

    Ok(())
}

/// Searches for Push/Pop instructions that handle an argument variable.
///
/// Argument variables are referenced using instance type `self` in pre-WAD15.
/// They are only differentiated from normal self variables by their name:
/// "argument0" - "argument15" are argument variables.
///
/// (At least from what info I found.)
fn get_arg_count(code: &Code, data: &GMData) -> Result<u8> {
    let mut max = 0;

    for instr in &code.instructions {
        let (Instruction::Pop { variable: codevar, .. }
        | Instruction::Push { value: PushValue::Variable(codevar) }) = instr
        else {
            continue;
        };
        if codevar.instance_type != InstanceType::Self_ {
            continue;
        }
        let var: &Variable = data.variables.by_ref(codevar.variable)?;
        let name: &String = data.strings.by_ref(var.name)?;
        let Some(num) = name.strip_prefix("argument") else {
            continue;
        };
        // hopefully LLVM can transform this into something that doesn't suck
        let Ok(num) = num.parse::<u8>() else { continue };
        if num > 15 {
            continue;
        }

        // println!(
        //     "Found argument variable {num} in instruction {}",
        //     crate::gml::assembly::disassemble_instruction(instr, data)?
        // );
        if num > max {
            max = num;
        }
    }

    Ok(max + 1)
}

/// Replaces this code in `gml_Script_SCR_TEXTTYPE`:
/// ```text
/// if (argument0 != 0) {
///     global.typer = argument0;
/// }
/// ```
/// with this code:
/// ```text
/// if (argument0 != undefined) {
///     global.typer = argument0;
/// }
///
/// In instructions:
/// ```
/// push.v self.argument0
/// push.e 0   # replace with `push.v self.undefined`
/// cmp.i.v NEQ
/// ```
fn fix_undertale(data: &mut GMData) {
    let Ok(script) = data.scripts.by_name("SCR_TEXTTYPE", &data.strings) else {
        return;
    };
    let Ok(code) = data.codes.by_ref_mut(script.code) else {
        return;
    };

    let Some(PushValue::Short(0)) = code.instructions.get(1).and_then(Instruction::push_value)
    else {
        return;
    };

    let undefined: GMRef<Variable> =
        data.variables.make("undefined", VariableType::Self_, data.meta.version, &mut data.strings);

    let codevar = CodeVariable::new(undefined, InstanceType::Self_);
    code.instructions[1] = Instruction::Push { value: PushValue::Variable(codevar) };
    log::debug!("Fixed SCR_TEXTTYPE argument0 undefined bug for Undertale");
}
