use std::collections::HashMap;
use crate::structs::DataChange;

#[derive(Clone)]
pub struct UTChunk {
    pub name: String,
    pub data: Vec<u8>,
    pub file_index: usize,
}

impl UTChunk {
    fn apply_changes(&self, mut changes: Vec<DataChange>) {
        changes.sort_by(|a, b| b.index.cmp(&a.index));
        for change in changes {
            println!(
                "[DataChange @ {}] Index: {} | Len: {} | Delete: {}",
                self.name,
                change.index,
                change.content.len(),
                change.delete
            );
            change.apply(self.data.clone());
        }
    }
    pub fn read_u64(&mut self) -> u64 {
        // Read unsigned 64-bit integer (little endian)
        let mut number: u64 = 0;
        for i in 0..8 {
            number |= u64::from(self.data[self.file_index]) << (i<<3);
            self.file_index += 1;
        }
        number
    }
    pub fn read_i64(&mut self) -> i64 {
        // Read signed 64-bit integer (little endian)
        let mut number: i64 = 0;
        for i in 0..8 {
            number |= i64::from(self.data[self.file_index]) << (i<<3);
            self.file_index += 1;
        }
        number
    }
    pub fn read_u32(&mut self) -> u32 {
        // Read unsigned 32-bit integer (little endian)
        let mut number: u32 = 0;
        for i in 0..4 {
            number |= u32::from(self.data[self.file_index]) << (i<<3);
            self.file_index += 1;
        }
        number
    }
    pub fn read_i32(&mut self) -> i32 {
        // Read signed 32-bit integer (little endian)
        let mut number: i32 = 0;
        for i in 0..4 {
            number |= i32::from(self.data[self.file_index]) << (i<<3);
            self.file_index += 1;
        }
        number
    }
    pub fn read_u16(&mut self) -> u16 {
        // Read unsigned 16-bit integer (little endian)
        let mut number: u16 = 0;
        for i in 0..2 {
            number |= u16::from(self.data[self.file_index]) << (i<<3);
            self.file_index += 1;
        }
        number
    }
    pub fn read_i16(&mut self) -> i16 {
        // Read signed 16-bit integer (little endian)
        let mut number: i16 = 0;
        for i in 0..2 {
            number |= i16::from(self.data[self.file_index]) << (i<<3);
            self.file_index += 1;
        }
        number
    }
    pub fn read_u8(&mut self) -> u8 {
        // Read unsigned 8-bit integer (little endian)
        let number: u8 = u8::from(self.data[self.file_index]);
        self.file_index += 1;
        number
    }
    pub fn read_i8(&mut self) -> i8 {
        // Read signed 8-bit integer (little endian)
        let number: i8 = self.data[self.file_index] as i8;
        self.file_index += 1;
        number
    }
    pub fn read_bool(&mut self) -> bool {
        self.read_u8() != 0
    }
    pub fn read_chunk_name(&mut self) -> String {
        let string: Vec<u8> = self.data[self.file_index..self.file_index + 4].to_owned();
        self.file_index += 4;
        let string: String = match String::from_utf8(string) {
            Ok(string) => string,
            Err(error) => {
                panic!("Invalid or corrupted data.win file (could not parse chunk name): {error}");
            }
        };
        string
    }
    pub fn read_literal_string(&mut self, length: usize) -> String {
        let string = self.data[self.file_index..self.file_index + length].to_owned();
        self.file_index += length;
        let string = match String::from_utf8(string) {
            Ok(string) => string,
            Err(error) => {
                panic!("Invalid or corrupted data.win file (could not parse string): {error}");
            }
        };
        string
    }

    pub fn read_ut_string(&mut self, ut_strings: &HashMap<u32, String>) -> String {
        let string_id: u32 = self.read_u32();
        let string: String = ut_strings[&string_id].clone();
        string
    }
}
