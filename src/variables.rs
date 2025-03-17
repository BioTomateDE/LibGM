use crate::chunk_reading::UTChunk;
use std::collections::HashMap;

pub struct UTVariable {
    pub name: String,
    pub instance_type: i32,
    pub variable_id: i32,
    pub occurrences_count: u32,
    pub first_occurrence_address: u32,
}

pub fn parse_chunk_VARI(mut chunk: UTChunk, strings: &HashMap<u32, String>) -> Vec<UTVariable> {
    let _unknown1: u32 = chunk.read_u32();
    let _unknown2: u32 = chunk.read_u32();
    let _unknown3: u32 = chunk.read_u32();
    let file_len: usize = chunk.data.len();
    let mut variables: Vec<UTVariable> = vec![];

    while chunk.file_index < file_len {
        variables.push(UTVariable {
            name: chunk.read_ut_string(strings),
            instance_type: chunk.read_i32(),
            variable_id: chunk.read_i32(),
            occurrences_count: chunk.read_u32(),
            first_occurrence_address: chunk.read_u32(),
        })
    }

    for var in &variables {
        if var.name == "arguments" {continue;}
        println!(
            "{:<20} {:<4} {:<4} {:<4} {}",
            var.name, var.instance_type, var.variable_id, var.occurrences_count, var.first_occurrence_address
        );
    }
    variables
}
