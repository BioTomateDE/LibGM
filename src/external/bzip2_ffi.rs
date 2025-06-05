use std::os::raw::{c_char, c_int};


// Link directly to libbzip2
#[link(name = "bz2")]
#[allow(non_snake_case)]
unsafe extern "C" {
    fn BZ2_bzBuffToBuffCompress(
        dest: *mut c_char,
        destLen: *mut u32,
        source: *const c_char,
        sourceLen: u32,
        blockSize100k: c_int,
        verbosity: c_int,
        workFactor: c_int,
    ) -> c_int;

    fn BZ2_bzBuffToBuffDecompress(
        dest: *mut c_char,
        destLen: *mut u32,
        source: *const c_char,
        sourceLen: u32,
        small: c_int,
        verbosity: c_int,
    ) -> c_int;
}

pub fn bz_compress(input: &[u8]) -> Result<Vec<u8>, String> {
    let block_size = 9; // Max compression (900k blocks)
    let mut output_len = (input.len() as f32 * 1.01) as u32 + 600; // bzip2's max formula

    let mut output = Vec::with_capacity(output_len as usize);
    let ret = unsafe { BZ2_bzBuffToBuffCompress(
        output.as_mut_ptr() as *mut c_char,
        &mut output_len,
        input.as_ptr() as *const c_char,
        input.len() as u32,
        block_size,
        0, // verbosity
        30, // workFactor
    ) };

    if ret != 0 {
        return Err(format!("BZip2 compression failed with code {ret}"))
    }
    unsafe { output.set_len(output_len as usize) };
    Ok(output)
}

pub fn bz_decompress(input: &[u8]) -> Result<Vec<u8>, String> {
    // For decompression, you'll need to know the expected size or handle growth
    let mut output_len: u32 = (input.len() * 10) as u32;    // Conservative estimate
    let mut output: Vec<u8> = Vec::with_capacity(output_len as usize);

    let ret = unsafe { BZ2_bzBuffToBuffDecompress(
        output.as_mut_ptr() as *mut c_char,
        &mut output_len as *mut u32,
        input.as_ptr() as *const c_char,
        input.len() as u32,
        0, // small
        0, // verbosity
    ) };

    if ret != 0 {
        return Err(format!("BZip2 decompression failed with code {ret}"))
    }
    unsafe { output.set_len(output_len as usize) };
    Ok(output)
}

