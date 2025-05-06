use std::collections::HashMap;
use std::io::Write;
use serde::Serialize;
use zip::write::{FileOptions, SimpleFileOptions};
use crate::deserialize::all::GMData;
use crate::export_mod::export::{ModWriter, FILE_OPTIONS};


#[derive(Debug, Clone, Serialize)]
struct ModStrings<'a> {
    add: &'a Vec<&'a String>,
    edit: &'a HashMap<usize, &'a String>,
}

fn export_strings(zip_writer: &mut ModWriter, data_orig: &GMData, data_mod: &GMData) -> Result<(), String> {
    let add: Vec<&String> = Vec::new();
    for string in data_orig.strings.strings_by_index {
        w
    }
    
    let mod_strings = ModStrings { add: &vec![], edit: Default::default() }
    
    zip_writer.start_file("strings.txt", *FILE_OPTIONS)
        .map_err(|e| format!("Failed to create strings file: {e}"))?;
    
    let string: String = serde_json::to_string(mod_strings)
        .map_err(|e| format!("Failed to serialize strings: {e}"))?;
    
    zip_writer.write_all(string.as_bytes())
        .map_err(|e| format!("Failed to write strings file: {e}"))?;

    Ok(())
}

