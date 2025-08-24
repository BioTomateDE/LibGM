#![deny(unused_must_use)]
#![deny(unreachable_patterns)]
#![deny(unused_assignments)]
#![deny(unused_macros)]
#![deny(clippy::all)]

#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]

mod gamemaker;
mod modding;
mod utility;
mod csharp_rng;

use std::path::Path;
use std::process::exit;
use crate::utility::Stopwatch;


fn read_data_file(data_file_path: &Path) -> Result<Vec<u8>, String> {
    let stopwatch = Stopwatch::start();
    let data: Vec<u8> = std::fs::read(data_file_path)
        .map_err(|e| format!("Could not read data file with path \"{}\": {e}", data_file_path.display()))?;
    log::trace!("Reading data file took {stopwatch}");
    Ok(data)
}

fn write_data_file(data: Vec<u8>, data_file_path: &Path) -> Result<(), String> {
    let stopwatch = Stopwatch::start();
    std::fs::write(data_file_path, data)
        .map_err(|e| format!("Could not write data file with path \"{}\": {e}", data_file_path.display()))?;
    log::trace!("Writing data file took {stopwatch}");
    Ok(())
}

fn path_from_arg<'a>(arg: Option<&'a String>, default: &'a str) -> &'a Path {
    Path::new(arg.map_or(default, |s| s))
}


fn main_open_and_close() -> Result<(), String> {
    use crate::gamemaker::data::GMData;
    use crate::gamemaker::deserialize::parse_data_file;
    use crate::gamemaker::serialize::build_data_file;
    let args: Vec<String> = std::env::args().collect();
    let original_data_file_path: &Path = path_from_arg(args.get(1), "data.win");
    let modified_data_file_path: &Path = path_from_arg(args.get(2), "data_out.win");

    log::info!("Loading data file \"{}\"", original_data_file_path.display());
    let original_data_raw: Vec<u8> = read_data_file(original_data_file_path)
        .map_err(|e| format!("{e}\n↳ while reading data file"))?;

    log::info!("Parsing data file");
    let mut original_data: GMData = parse_data_file(&original_data_raw, false)
        .map_err(|e| format!("\n{e}\n↳ while parsing data file"))?;
    drop(original_data_raw);
    
    // // sample changes
    // let original_name: &str = original_data.general_info.display_name.resolve(&original_data.strings.strings)?;
    // let modified_name: String = format!("{original_name} - Modded using AcornGM");
    // original_data.general_info.display_name = original_data.make_string(&modified_name);
    
    // // export code disassembly
    // for code in &original_data.codes.codes {
    //     let code_name = code.name.resolve(&original_data.strings.strings)?;
    //     let assembly = crate::gamemaker::disassembler::disassemble_code(&original_data, code)?;
    //     // println!("Disassembly of \"{code_name}\": \n{}", assembly);
    //     std::fs::write(format!("./gml_asm/{code_name}.txt"), assembly)
    //         .map_err(|e| format!("Could not write assembly of code \"{code_name}\": {e}"))?;
    // }
    
    // test (dis)assembler
    let mut assemblies = Vec::with_capacity(original_data.codes.codes.len());

    for (i, code) in original_data.codes.codes.iter().enumerate() {
        if code.bytecode15_info.as_ref().map_or(false, |b15| b15.parent.is_some()) {
            continue    // code is child entry; FUCK THAT CHILD
        }
        let code_name = code.name.resolve(&original_data.strings.strings)?;
        let mut locals = None;
        for code_local in &original_data.functions.code_locals.code_locals {
            let code_local_name = code_local.name.resolve(&original_data.strings.strings)?;
            if *code_local_name == *code_name {
                locals = Some(code_local);
                break
            }
        }
        // println!("{code_name}");
        let locals = locals.ok_or("Couldn't find locals")?;
        let assembly = gamemaker::disassembler::disassemble_code(&original_data, code)?;
        assemblies.push((i, code_name.clone(), locals.clone(), assembly));
    }

    // println!("\n=============================\n");
    for (i, name, locals, assembly) in assemblies {
        println!("{i:<4} {name}");
        let reconstructed = gamemaker::assembler::assemble_code(&assembly, &mut original_data, &locals).map_err(|e| e.to_string())?;
        if original_data.codes.codes[i].instructions != reconstructed {
            return Err(format!(
                "Reconstructed instructions don't match original for {name}:\n##{}##\n\n\n\n\n##{}##",
                original_data.codes.codes[i].instructions.iter().map(|i| format!("{i:?}")).collect::<Vec<_>>().join("\n"),
                reconstructed.iter().map(|i| format!("{i:?}")).collect::<Vec<_>>().join("\n"),
            ))
        }
    }

    // // export strings
    // let mut raw = String::new();
    // for i in 0..original_data.strings.strings.len() {
    //     let string_ref = gamemaker::deserialize::GMRef::new(i as u32);
    //     let string = gamemaker::disassembler::format_literal_string(&original_data, string_ref)?;
    //     raw += &string;
    //     raw += "\n";
    // }
    // std::fs::write(format!("{}_strings.txt", original_data_file_path.to_str().unwrap()), raw)
    //     .map_err(|e| format!("Could not write string: {e}"))?;

    log::info!("Building data file");
    let modified_data_raw: Vec<u8> = build_data_file(&original_data)
        .map_err(|e| format!("\n{e}\n↳ while building data file"))?;
    drop(original_data);

    log::info!("Writing data file \"{}\"", modified_data_file_path.display());
    write_data_file(modified_data_raw, modified_data_file_path)
        .map_err(|e| format!("{e}\n↳ while writing data file"))?;

    Ok(())
}


// fn main_export_mod() -> Result<(), String> {
//     use crate::modding::export::{export_mod};
//     use crate::gamemaker::deserialize::{parse_data_file, GMData};
//     let args: Vec<String> = std::env::args().collect();
//     let original_data_file_path = path_from_arg(args.get(1), "data_original.win");
//     let modified_data_file_path = path_from_arg(args.get(2), "data_modified.win");
//     let mod_data_path = path_from_arg(args.get(3), "acornmod.tar.zst");
// 
//     log::info!("Loading original data file \"{}\"", original_data_file_path.display());
//     let original_data_raw: Vec<u8> = read_data_file(original_data_file_path)
//         .map_err(|e| format!("{e}\n↳ while reading original data file"))?;
// 
//     log::info!("Parsing original data file");
//     let original_data: GMData = parse_data_file(&original_data_raw, false)
//         .map_err(|e| format!("{e}\n↳ while parsing original data file"))?;
//     drop(original_data_raw);
// 
//     log::info!("Loading modified data file \"{}\"", modified_data_file_path.display());
//     let modified_data_raw: Vec<u8> = read_data_file(modified_data_file_path)
//         .map_err(|e| format!("{e}\n↳ while reading modified data file"))?;
// 
//     log::info!("Parsing modified data file");
//     let modified_data: GMData = parse_data_file(&modified_data_raw, false)
//         .map_err(|e| format!("{e}\n↳ while parsing modified data file"))?;
//     drop(modified_data_raw);
// 
//     log::info!("Extracting changes and exporting mod to file \"{}\"", mod_data_path.display());
//     export_mod(&original_data, &modified_data, mod_data_path)
//         .map_err(|e| format!("{e}\n↳ while exporting AcornGM mod"))?;
// 
//     Ok(())
// }


fn main() {
    biologischer_log::init(env!("CARGO_PKG_NAME"));
    log::debug!("============= LibGM v{} =============", env!("CARGO_PKG_VERSION"));
    
    if let Err(e) = main_open_and_close() {
        log::error!("{e}");
        exit(1);
    }

    log::info!("Done");
}

