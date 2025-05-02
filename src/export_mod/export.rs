use crate::deserialize::all::GMData;
use std::fs::File;
use std::io::prelude::*;
use std::io::Cursor;
use std::sync::LazyLock;
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


// add
// edit
// insert
// remove
