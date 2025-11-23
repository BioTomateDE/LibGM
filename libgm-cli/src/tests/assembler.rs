use libgm::gamemaker::data::GMData;
use libgm::gml::assembly::{assemble_code, disassemble_code};
use libgm::gml::instructions::GMInstruction;
use libgm::prelude::*;
use std::io::Write;

pub fn test_assembler(data: &GMData) -> Result<()> {
    let count = data.codes.len();

    for i in 0..count {
        let code = &data.codes[i];

        // Skip child code entries.
        if let Some(b15) = &code.bytecode15_info
            && b15.parent.is_some()
        {
            continue;
        }

        let name = &code.name;
        print!("\x1B[2K\r({i}/{count}) Disassembling {name}");
        std::io::stdout().flush().unwrap();

        let assembly: String = disassemble_code(data, code)
            .with_context(|| format!("disassembling {name:?}"))?;

        let reconstructed: Vec<GMInstruction> = assemble_code(&assembly, data)
            .with_context(|| format!("assembling {name:?}"))?;

        let code = &data.codes[i];
        if code.instructions == reconstructed {
            continue;
        }

        // Assembler (or dissassembler) failed; produced different instructions.
        let orig_len = code.instructions.len();
        let recr_len = reconstructed.len();

        if recr_len != orig_len {
            let diff = recr_len.abs_diff(orig_len);
            let comparison = if recr_len > orig_len { "more" } else { "fewer" };
            println!(
                "Reconstructed code has {diff} {comparison} instructions than the original"
            );
        }

        for (original, recreation) in
            code.instructions.iter().zip(&reconstructed)
        {
            if original != recreation {
                println!(
                    "Original: {original:?}\nRecreation: {recreation:?}\n"
                );
            }
        }

        return Err(
            "Assembler produced different instructions than the original (see logs)".into(),
        );
    }

    Ok(())
}
