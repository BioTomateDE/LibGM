use std::cmp::min;
use std::collections::HashMap;
use crate::deserialize::all::GMData;
use std::fs::File;
use std::io::prelude::*;
use std::io::Cursor;
use std::sync::LazyLock;
use serde_json::json;
use zip::write::{FileOptions, SimpleFileOptions};
use zip::{CompressionMethod, ZipWriter};

pub type ModWriter<'a> = ZipWriter<Cursor<&'a mut Vec<u8>>>;
pub const FILE_OPTIONS: LazyLock<FileOptions<()>> = LazyLock::new(|| 
    SimpleFileOptions::default()
    .compression_method(CompressionMethod::Bzip2)
    .compression_level(Some(9))
);

fn export_mod(gm_data: &GMData) {
    let mut data: Vec<u8> = Vec::new();
    let buff = Cursor::new(&mut data);
    let mut zip_writer = ZipWriter::new(buff);
    zip_writer.start_file("some.txt", SimpleFileOptions::default()).unwrap();
    zip_writer.write_all(b"hello").unwrap();
    zip_writer.finish().unwrap();
    let mut file = File::create("foo.zip").unwrap();
    file.write_all(data.as_slice()).unwrap();
}


// fn export_unordered_list<G, A>(orig: &Vec<G>, modded: &Vec<G>, map_add: fn(&G) -> A, map_edit: fn(&G, &G) -> A) -> Result<serde_json::Value, String> {
//     let additions: Vec<A> = modded.get(orig.len() .. modded.len())
//         .ok_or_else(|| format!("Could not get {G} additions slice with orig len {} and modded len {}", orig.len(), modded.len()))?
//         .iter().map(map_add).collect();
//
//     let mut edits: HashMap<usize, A> = HashMap::new();
//     for i in 0..min(orig.len(), modded.len()) {
//         if orig[i] == modded[i] {
//             continue
//         }
//         edits.insert(i, map_edit(&orig[i], &modded[i]));
//     }
//
//     Ok(json!({
//         "add": additions,
//         "edit": edits,
//     }))
// }
// TODO


// add
// edit
// insert
// remove
