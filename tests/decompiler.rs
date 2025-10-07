use libgm::prelude::*;

#[test]
fn test_decompiler() -> Result<()> {
    use libgm::__test_data_files;
    use libgm::gamemaker::deserialize::GMRef;
    use libgm::gml::decompiler::decompile_to_ast;

    __test_data_files(|data| {
        for (i, code) in data.codes.codes.iter().enumerate() {
            if let Some(b15) = &code.bytecode15_info {
                if b15.parent.is_some() {
                    continue;
                }
            }

            let code_name = code.name.resolve(&data.strings.strings)?;
            print!(
                "({}/{}) Decompiling: {:<64}\n",
                i + 1,
                data.codes.codes.len(),
                code_name
            );
            decompile_to_ast(&data, GMRef::new(i as u32))?;
        }
        Ok(())
    })
}
