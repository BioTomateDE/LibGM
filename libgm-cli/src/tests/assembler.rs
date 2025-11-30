use std::io::Write;

use libgm::{
    gamemaker::data::GMData,
    gml::{
        assembly::{assemble_code, disassemble_code},
        instructions::GMInstruction,
    },
    prelude::*,
};

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
        //print!("\x1B[2K\r({i}/{count}) Disassembling {name}");
        //std::io::stdout().flush().unwrap();
        //
        println!("({i}/{count}) Disassembling {name:?}");

        let assembly: String =
            disassemble_code(data, code).with_context(|| format!("disassembling {name:?}"))?;

        let reconstructed: Vec<GMInstruction> =
            assemble_code(&assembly, data).with_context(|| format!("assembling {name:?}"))?;

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
            println!("Reconstructed code has {diff} {comparison} instructions than the original");
        }

        let lines: Vec<&str> = assembly.split("\n").collect();

        for (index, (original, recreation)) in
            code.instructions.iter().zip(&reconstructed).enumerate()
        {
            if original != recreation {
                let line = lines[index];
                println!("Original: {original:?}");
                println!("Recreation: {recreation:?}");
                println!("Assembly: {line}");
                println!();
            }
        }

        return Err(
            "Assembler produced different instructions than the original (see logs)".into(),
        );
    }
    println!();
    Ok(())
}
