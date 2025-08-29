
#[test]
fn test_code_validation() {
    use libgm::gamemaker::code_related::validation::validate_code;
    use libgm::__test_data_files;

    __test_data_files(|data| {
        for (i, code) in data.codes.codes.iter().enumerate() {
            let code_name = code.name.resolve(&data.strings.strings)?;
            print!("({}/{}) Validating: {:<64}\n", i+1, data.codes.codes.len(), code_name);
            validate_code(code, &data.codes).map_err(|e| e.to_string())?;
        }
        Ok(())
    })
}
