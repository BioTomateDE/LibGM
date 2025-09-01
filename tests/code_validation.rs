
#[test]
fn test_code_validation() -> Result<(), String> {
    use libgm::gamemaker::code_related::validation::validate_code;
    use libgm::__test_data_files;

    __test_data_files(|data| {
        //todo remove debuug
        for (i, code) in data.codes.codes[34..].iter().enumerate() {
            let code_name = code.name.resolve(&data.strings.strings)?;
            print!("({}/{}) Validating: {:<64}\n", i+1, data.codes.codes.len(), code_name);
            validate_code(code, &data).map_err(|e| format!("{e}\n↳ while validating code {code_name}"))?;
        }
        Ok(())
    })
}
