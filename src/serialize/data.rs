use std::fs;
use crate::chunk_reading::UTChunk;
use crate::chunk_writing::DataBuilder;
use crate::structs::UTData;

use crate::serialize::strings::build_chunk_STRG;


pub fn build_data_file(ut_data: &UTData) -> Result<Vec<u8>, String> {
    let mut builder: DataBuilder = DataBuilder { raw_data: Vec::new() };

    let chunk_STRG: UTChunk = build_chunk_STRG(&ut_data)?;


    builder.write_string("FORM")?;



    Ok(builder.raw_data)
}


pub fn write_data_file(data_file_path: &str, raw_data: &[u8]) -> Result<(), String> {
    match fs::write(data_file_path, raw_data) {
        Ok(_) => Ok(()),
        Err(error) => {
            Err(format!("Could not write to data file: {error}"))
        }
    }
}

