// SPDX-License-Identifier: GPL-3.0-only
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use libgm::gml::GMCode;
use libgm::gml::Instruction;
use libgm::gml::assembly::assemble_instructions;
use libgm::gml::assembly::disassemble_code;
use libgm::prelude::*;
use libgm::wad::build_bytes;
use libgm::wad::parse_bytes;
use libgm::wad::parse_file;
use libgm_cli::diff::print_diff;
use libgm_cli::diff::print_diffs;
use sha2::Digest;
use sha2::Sha256;

fn resolve_path(filename: &str) -> PathBuf {
    let filename = Path::new(filename);
    if filename.is_absolute() {
        panic!("Invalid filename {}", filename.display());
    }

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("datafiles");
    path.push(filename);
    path
}

// TODO: support multiple possible hashes for all sets of valid files
fn verify_integrity(path: &Path, hash: &str) -> Result<()> {
    let data = fs::read(path).ctx_any(|| format!("reading data file {}", path.display()))?;

    // TODO: ugly
    let expected = hash
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|byte| u8::from_str_radix(str::from_utf8(byte).unwrap(), 0x10).unwrap())
        .collect::<Vec<_>>();

    let actual = Sha256::digest(data);
    if *expected == *actual {
        return Ok(());
    }

    Err(err!(
        "Integrity check failed for {}: Expected SHA256 hash {:?} did not match actual hash {:?}",
        path.display(),
        expected,
        actual,
    ))
}

fn check_reparse(data: &GMData) -> Result<()> {
    let raw_data = build_bytes(data).ctx("building data for reparse")?;
    let reparsed_data = parse_bytes(raw_data).ctx("reparsing data")?;

    if print_diffs(data, &reparsed_data) {
        bail!("Reparsing produced different data!");
    }

    Ok(())
}

fn reassemble_one(data: &mut GMData, code: &GMCode) -> Result<()> {
    let assembly: String = disassemble_code(code, data)?;
    let reconstructed: Vec<Instruction> = assemble_instructions(&assembly, data)?;

    if code.instructions == reconstructed {
        return Ok(());
    }

    print_diff(&code.instructions, &reconstructed);
    Err(err!(
        "Reassembly of {} (original: left) produced different instructions (right)",
        data.strings.by_ref(code.name)?
    ))
}

fn check_reassemble(data: &mut GMData) -> Result<()> {
    // too lazy to fix rn
    for code in data.codes.elements() {
        if code.is_root() {
            reassemble_one(data, code)
                .ctx(|| format!("disassembling and reassembling code entry {:?}", code.name))?;
        }
    }

    Ok(())
}

fn perform_inner(data_file_path: &Path, sha256sum: &str) -> Result<()> {
    verify_integrity(data_file_path, sha256sum)?;
    let mut data = parse_file(data_file_path).ctx("parsing data file")?;
    data.validate_names().ctx("validating all names")?;
    check_reparse(&data)?;
    check_reassemble(&mut data)?;
    Ok(())
}

#[allow(dead_code)]
fn perform(data_file_name: &str, sha256sum: &str) {
    let path = resolve_path(data_file_name);
    // if !path.exists() {
    //     eprintln!("Skipping test for non-existent data file {data_file_name:?}");
    //     return;
    // }
    if let Err(e) = perform_inner(&path, sha256sum) {
        eprintln!("{}", e.chain_pretty());
        panic!("End-To-End Test failed for {data_file_name}");
    }
}

#[cfg(feature = "test-undertale-100")]
#[test]
fn undertale100() {
    perform(
        "undertale100.win",
        "7f3e3d6ddc5e6ba3bd45f94c1d6277becbbf3a519d1941d321289d7d2b9f5d26",
    )
}

#[cfg(feature = "test-undertale-101")]
#[test]
fn undertale101() {
    perform(
        "undertale101.win",
        "3f85bc6204c2bf4975515e0f5283f5256e2875c81d8746db421182abd7123b08",
    )
}

#[cfg(feature = "test-undertale-108")]
#[test]
fn undertale108() {
    perform(
        "undertale108.win",
        "b718f8223a5bb31979ffeed10be6140c857b882fc0d0462b89d6287ae38c81c7",
    )
}

#[cfg(feature = "test-deltarune-ch1234")]
#[test]
fn deltarune_launcher() {
    perform(
        "deltarune-launcher.win",
        "e1e2c38d250fba7f5387abb773bf2716b5783708b93634c4c1b162f1f1790361",
    );
}

#[cfg(feature = "test-deltarune-ch1234")]
#[test]
fn deltarune_ch1() {
    perform(
        "deltarune1.win",
        "fd410845c40cd084eb33a3410bfdb790cc0ad9f938fe088420ca57774c0dfcb7",
    );
}

#[cfg(feature = "test-deltarune-ch1234")]
#[test]
fn deltarune_ch2() {
    perform(
        "deltarune2.win",
        "3fdd81e30723e5dfde2309c349600ce92a2305a706d33ae9edeacbcb20c6ce3c",
    );
}

#[cfg(feature = "test-deltarune-ch1234")]
#[test]
fn deltarune_ch3() {
    perform(
        "deltarune3.win",
        "83b49c0a2939a6cabefd024f5c2c7d8c08df242d3bf7a725a4f2bbe97a5f081d",
    );
}

#[cfg(feature = "test-deltarune-ch1234")]
#[test]
fn deltarune_ch4() {
    perform(
        "deltarune4.win",
        "07e2df1088e56532b992fc9c59f88a4d66420ada4da9ff49ba6823a7f2cc3d47",
    );
}
