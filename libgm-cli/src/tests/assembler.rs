// SPDX-License-Identifier: GPL-3.0-only
use libgm::gml::assembly::assemble_instructions;
use libgm::gml::assembly::disassemble_code;
use libgm::gml::instruction::Instruction;
use libgm::prelude::*;
use libgm::wad::data::GMData;
use libgm_cli::diff::print_diff;

pub fn test(data: &mut GMData) -> Result<()> {
    for i in 0..data.codes.len() {
        let code_ref = GMRef::from(i);
        let code = data.codes.by_ref(code_ref)?;
        let name = data.strings.by_ref(code.name)?;

        // Skip child code entries.
        if let Some(data) = &code.modern_data
            && data.parent.is_some()
        {
            continue;
        }

        let name_ref = code.name;
        let assembly: String =
            disassemble_code(code, data).ctx(|| format!("disassembling {name:?}"))?;
        let reconstructed: Vec<Instruction> = assemble_instructions(&assembly, data)
            .ctx(|| format!("assembling {:?}", data.strings.by_ref(name_ref).unwrap()))?;

        // the borrow checker is being annoying again :(
        let code = data.codes.by_ref(code_ref)?;
        let name = data.strings.by_ref(code.name)?;
        if code.instructions == reconstructed {
            continue;
        }

        // Assembler (or disassembler) failed; produced different instructions.
        print_diff(&code.instructions, &reconstructed);
        bail!("Roundtrip validation failed for {name:?}, see logs");
    }

    println!();
    Ok(())
}
