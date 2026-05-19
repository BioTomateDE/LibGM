use std::fs;
use std::iter;
use std::path::Path;
use std::path::PathBuf;

use libgm::gml::assembly::assemble_instructions;
use libgm::gml::assembly::disassemble_code;
use libgm::gml::Instruction;
use libgm::wad::build_bytes;
use libgm::wad::parse_bytes;
use libgm::wad::GMData;
use libgm_cli::diff::print_diff;
use libgm_cli::diff::print_diffs;
use sha2::Digest;
use sha2::Sha256;
use snafu::ResultExt;
use snafu::Whatever;

fn resolve_path(start: &str) -> PathBuf {
    let parent = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut result = parent.to_owned();
    let mut popped_filename = false;
    for component in Path::new(start) {
        if component == ".." {
            if popped_filename {
                result.pop();
            } else {
                popped_filename = true;
            }
        } else {
            result.push(component);
        }
    }
    result
}

fn verify_integrity(path: &Path, hash: &str) -> Result<(), Whatever> {
    let data = fs::read(path)
        .whatever_context(format!("Failed to read contents of {}", path.display()))?;
    let expected = hash
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|byte| u8::from_str_radix(str::from_utf8(byte).unwrap(), 0x10).unwrap())
        .collect::<Vec<_>>();
    let actual = Sha256::digest(data);
    assert_eq!(
        expected.as_slice(),
        actual.as_slice(),
        "{} is invalid: expected sha256 hash {expected:?} did not match actual {actual:?}",
        path.display()
    );
    Ok(())
}

fn check_reparse(data: &GMData) -> Result<(), Whatever> {
    let raw_data = build_bytes(data).whatever_context("Failed to convert GMData to bytes")?;
    let reparsed_data =
        parse_bytes(raw_data).whatever_context("Failed to reparse bytes as GMData")?;

    if !print_diffs(data, &reparsed_data) {
        panic!("Reparse produced different data");
    }

    Ok(())
}

fn check_reassemble(data: &GMData) -> Result<(), Whatever> {
    for code in &data.codes {
        if code.is_root() {
            let assembly: String = disassemble_code(code, data)
                .whatever_context(format!("Failed to disassemble {}", code.name))?;

            let reconstructed: Vec<Instruction> = assemble_instructions(&assembly, data)
                .whatever_context(format!("Failed to reassemble {}", code.name))?;

            if code.instructions != reconstructed {
                print_diff(&code.instructions, &reconstructed);
                panic!(
                    "Reassembly of {} (original: left) produced different instructions (right)",
                    code.name
                );
            }
        }
    }

    Ok(())
}

// TODO: support multiple possible hashes for all sets of valid files
macro_rules! e2e {
    ($(#[$attr:meta])* $name:ident, path: $path:literal, hash: $hash:literal $(,)?) => {
        $(#[$attr])*
        #[snafu::report]
        #[test]
        fn $name() -> Result<(), snafu::Whatever> {
            let path = resolve_path($path);

            verify_integrity(&path, $hash)
                .whatever_context("Invalid file for required version")?;

            let parser = libgm::wad::deserialize::ParsingOptions::new()
                .verify_alignment(true)
                .verify_constants(true)
                .allow_unknown_chunks(false);

            let data = parser
                .parse_file(path)
                .whatever_context("Failed to parse file as GMData")?;

            data.validate_names()
                .whatever_context("Failed to validate names")?;

            check_reparse(&data)?;

            check_reassemble(&data)?;

            Ok(())
        }
    };
}

e2e! {
    #[cfg(feature = "test-undertale-1_0_8win")]
    undertale_e2e,
    path: "../resources/undertale.win",
    hash: "b718f8223a5bb31979ffeed10be6140c857b882fc0d0462b89d6287ae38c81c7"
}

e2e! {
    #[cfg(feature = "test-deltarune-1to4-1_0_4win")]
    deltarune_base_e2e,
    path: "../resources/deltarune.win",
    hash: "e1e2c38d250fba7f5387abb773bf2716b5783708b93634c4c1b162f1f1790361"
}

e2e! {
    #[cfg(feature = "test-deltarune-1to4-1_0_4win")]
    deltarune_ch1_e2e,
    path: "../resources/deltarune_ch1.win",
    hash: "fd410845c40cd084eb33a3410bfdb790cc0ad9f938fe088420ca57774c0dfcb7"
}

e2e! {
    #[cfg(feature = "test-deltarune-1to4-1_0_4win")]
    deltarune_ch2_e2e,
    path: "../resources/deltarune_ch2.win",
    hash: "3fdd81e30723e5dfde2309c349600ce92a2305a706d33ae9edeacbcb20c6ce3c"
}

e2e! {
    #[cfg(feature = "test-deltarune-1to4-1_0_4win")]
    deltarune_ch3_e2e,
    path: "../resources/deltarune_ch3.win",
    hash: "83b49c0a2939a6cabefd024f5c2c7d8c08df242d3bf7a725a4f2bbe97a5f081d"
}

e2e! {
    #[cfg(feature = "test-deltarune-1to4-1_0_4win")]
    deltarune_ch4_e2e,
    path: "../resources/deltarune_ch4.win",
    hash: "07e2df1088e56532b992fc9c59f88a4d66420ada4da9ff49ba6823a7f2cc3d47"
}
