
#[test]
fn test_disassembler_and_assembler() -> Result<(), String> {
    use libgm::gamemaker::code_related::assembler::assemble_code;
    use libgm::gamemaker::code_related::disassembler::disassemble_code;
    use libgm::gamemaker::elements::code::GMInstruction;
    use libgm::__test_data_files;

    __test_data_files(|mut data| {
        for i in 0..data.codes.codes.len() {
            let code = &data.codes.codes[i];
            if code.bytecode15_info.as_ref().map_or(false, |b15| b15.parent.is_some()) {
                continue    // code is child entry; skip
            }

            let code_name = code.name.resolve(&data.strings.strings)?;
            print!("({}/{}) Disassembling: {:<64}\r", i+1, data.codes.codes.len(), code_name);

            let assembly: String = disassemble_code(&data, code)?;
            let reconstructed: Vec<GMInstruction> = assemble_code(&assembly, &mut data).map_err(|e| e.to_string())?;

            let code = &data.codes.codes[i];
            if code.instructions != reconstructed {
                for (original, recreation) in code.instructions.iter().zip(reconstructed) {
                    if *original != recreation {
                        log::error!("Original: {:?}\nRecreation: {:?}\n", original, recreation);
                    }
                }
                return Err("Assembler produced different instructions than the original (see logs)".to_string())
            }
        }
        Ok(())
    })
}
