use libgm::prelude::*;

#[test]
fn test_decompiler() {
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

            if code.instructions.is_empty() {
                continue;
            }

            let game = if data.general_info.version.major == 1 {
                "Undertale"
            } else if data.general_info.version.major == 2022 {
                "DeltaruneDemo"
            } else if data.strings.strings.iter().any(|i| i == "DELTARUNE Chapter 1") {
                "Chapter1"
            } else if data.strings.strings.iter().any(|i| i == "DELTARUNE Chapter 2") {
                "Chapter2"
            } else if data.strings.strings.iter().any(|i| i == "DELTARUNE Chapter 3") {
                "Chapter3"
            } else if data.strings.strings.iter().any(|i| i == "DELTARUNE Chapter 4") {
                "Chapter4"
            } else {
                "DeltaruneLauncher"
            };
            unsafe { std::env::set_var("FUCKING_GAMENAME", game) }
            // TODO remove debug
            decompile_to_ast(&data, GMRef::new(i as u32)).with_context(|| format!("decompiling {code_name}"))?;
        }
        Ok(())
    })
}
