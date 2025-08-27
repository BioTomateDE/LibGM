
#[test]
fn test_disassembler_and_assembler() {
    use libgm::gamemaker::assembler::assemble_code;
    use libgm::gamemaker::disassembler::disassemble_code;
    use libgm::gamemaker::elements::code::GMInstruction;
    use libgm::gamemaker::elements::functions::GMCodeLocal;
    use libgm::__test_data_files;

    __test_data_files(|mut data| {
        for i in 0..data.codes.codes.len() {
            let code = &data.codes.codes[i];
            if code.bytecode15_info.as_ref().map_or(false, |b15| b15.parent.is_some()) {
                continue    // code is child entry; skip
            }

            let code_name = code.name.resolve(&data.strings.strings)?;
            let mut locals = None;
            for code_local in &data.functions.code_locals.code_locals {
                let code_local_name = code_local.name.resolve(&data.strings.strings)?;
                if *code_local_name == *code_name {
                    locals = Some(code_local);
                    break
                }
            }
            let locals: GMCodeLocal = locals.ok_or("Couldn't find locals")?.clone();

            print!("Disassembling ({}/{}): {:<64}\r", i, data.codes.codes.len(), code_name);
            let assembly: String = disassemble_code(&data, code)?;
            let reconstructed: Vec<GMInstruction> = assemble_code(&assembly, &mut data, &locals).map_err(|e| e.to_string())?;

            let code = &data.codes.codes[i];
            if code.instructions != reconstructed {
                for (original, recreation) in code.instructions.iter().zip(reconstructed) {
                    if *original != recreation {
                        log::error!("Original: {:?}\nRecreation: {:?}\n", original, recreation);
                    }
                }
                return Err("Assembler produced different instructions than the original".to_string())
            }
        }
        Ok(())
    })
}
