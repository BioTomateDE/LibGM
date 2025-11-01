use libgm::gamemaker::elements::code::GMInstruction;
use libgm::gml::assembler::assemble_code;
use libgm::gml::disassembler::disassemble_code;
use libgm::prelude::*;
use libgm::test::test_data_files;

#[test]
fn test_assembler() {
    test_data_files(|mut data| {
        let count = data.codes.codes.len();

        for i in 0..count {
            let code = &data.codes.codes[i];

            // Skip child code entries.
            if let Some(b15) = &code.bytecode15_info {
                if b15.parent.is_some() {
                    continue;
                }
            }

            let code_name = code.name.resolve(&data.strings.strings)?.clone();
            print!("({}/{}) Disassembling: {:<64}\r", i + 1, count, code_name);

            let assembly: String =
                disassemble_code(&data, code).with_context(|| format!("disassembling {code_name:?}"))?;

            let reconstructed: Vec<GMInstruction> =
                assemble_code(&assembly, &mut data).with_context(|| format!("assembling {code_name:?}"))?;

            let code = &data.codes.codes[i];
            if code.instructions == reconstructed {
                continue;
            }

            // Assembler (or dissassembler) failed; produced different instructions.
            let orig_len = code.instructions.len();
            let recr_len = reconstructed.len();

            if recr_len != orig_len {
                let diff = recr_len.abs_diff(orig_len);
                let comparison = if recr_len > orig_len { "more" } else { "fewer" };
                log::error!("Reconstructed code has {diff} {comparison} instructions than the original");
            }

            for (original, recreation) in code.instructions.iter().zip(&reconstructed) {
                if original != recreation {
                    log::error!("Original: {original:?}\nRecreation: {recreation:?}\n");
                }
            }

            bail!("Assembler produced different instructions than the original (see logs)");
        }
        Ok(())
    })
}
