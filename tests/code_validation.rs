use libgm::prelude::*;

#[test]
fn test_code_validation() {
    use libgm::__test_data_files;
    use libgm::gml::validation::validate_code;

    __test_data_files(|data| {
        for (i, code) in data.codes.codes.iter().enumerate() {
            let code_name = code.name.resolve(&data.strings.strings)?;
            print!("({}/{}) Validating: {:<64}\n", i + 1, data.codes.codes.len(), code_name);
            validate_code(code, &data).with_context(|| format!("validating code {code_name}"))?;
        }
        Ok(())
    })
}
