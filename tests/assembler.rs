use libgm::prelude::*;

#[test]
fn test_disassembler_and_assembler() {
    use libgm::__test_data_files;
    use libgm::gamemaker::elements::code::GMInstruction;
    use libgm::gml::assembler::assemble_code;
    use libgm::gml::disassembler::disassemble_code;

    __test_data_files(|mut data| {
        for i in 0..data.codes.codes.len() {
            let code = &data.codes.codes[i];
            if code.bytecode15_info.as_ref().map_or(false, |b15| b15.parent.is_some()) {
                continue; // Code is child entry; skip
            }

            let code_name = code.name.resolve(&data.strings.strings)?.clone();
            print!(
                "({}/{}) Disassembling: {:<64}\r",
                i + 1,
                data.codes.codes.len(),
                code_name
            );

            let assembly: String =
                disassemble_code(&data, &code).with_context(|| format!("disassembling {code_name:?}"))?;

            let reconstructed: Vec<GMInstruction> =
                assemble_code(&assembly, &mut data).with_context(|| format!("assembling {code_name:?}"))?;

            let code = &data.codes.codes[i];
            if code.instructions != reconstructed {
                let ori_len = code.instructions.len();
                let rec_len = reconstructed.len();

                if rec_len != ori_len {
                    let diff = rec_len.abs_diff(ori_len);
                    let comparison = if rec_len > ori_len { "more" } else { "fewer" };
                    log::error!("Reconstructed code has {diff} {comparison} instructions than the original");
                }

                for (original, recreation) in code.instructions.iter().zip(&reconstructed) {
                    if original != recreation {
                        log::error!("Original: {original:?}\nRecreation: {recreation:?}\n");
                    }
                }
                bail!("Assembler produced different instructions than the original (see logs)");
            }
        }
        Ok(())
    })
}
