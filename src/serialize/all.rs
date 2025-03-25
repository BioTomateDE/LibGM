use std::fs;
use crate::deserialize::all::UTData;
use crate::deserialize::chunk_reading::UTChunk;
use crate::serialize::data_writing::DataBuilder;

use crate::serialize::strings::build_chunk_STRG;
use crate::serialize::general_info::{build_chunk_OPTN, build_chunk_GEN8};


pub fn build_data_file(ut_data: &UTData) -> Result<Vec<u8>, String> {
    let mut builder: DataBuilder = DataBuilder { raw_data: Vec::new() };
    let chunks: Vec<UTChunk> = vec![
        build_chunk_GEN8(&ut_data)?,
        build_chunk_OPTN(&ut_data)?,
        // build_chunk_EXTN(&ut_data)?,
        // build_chunk_SOND(&ut_data)?,
        build_chunk_STRG(&ut_data)?,
    ];

    let mut total_len: usize = 0;
    for chunk in &chunks {
        total_len += chunk.data_len;
    }
    builder.write_string("FORM")?;
    builder.write_usize(total_len)?;

    for chunk in &chunks {
        builder.write_string(&chunk.name)?;
        builder.write_usize(chunk.data_len)?;
        builder.raw_data.extend(&chunk.data);
    }


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

