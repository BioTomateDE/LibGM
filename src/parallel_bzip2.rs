use std::ffi::{c_char, c_int};
use rayon::prelude::*;
use std::sync::{Arc, Mutex, MutexGuard};
use rayon::ThreadPool;

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

unsafe fn compress_chunk(input: &[u8]) -> Vec<u8> {
    let block_size: i32 = 9;     // max compression (900k blocks)
    let mut output_len: u32 = (input.len() as f32 * 1.01) as u32 + 600;   // bzip2's max formula

    let mut output = Vec::with_capacity(output_len as usize);
    let ret = BZ2_bzBuffToBuffCompress(
        output.as_mut_ptr() as *mut c_char,
        &mut output_len,
        input.as_ptr() as *const c_char,
        input.len() as u32,
        block_size,
        0,    // verbosity
        30, // workFactor
    );

    if ret != 0 {
        panic!("BZip2 compression failed with error code {ret}")
    }
    output.set_len(output_len as usize);
    output
}


pub fn compress_parallel(input: &[u8], chunk_size: usize) -> Result<Vec<u8>, String> {
    let num_threads: usize = optimal_thread_count(input.len());
    let pool: ThreadPool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .map_err(|e| format!("Could not build rayon ThreadPool: {e}"))?;

    let chunks: Vec<&[u8]> = input.chunks(chunk_size).collect::<Vec<_>>();
    let compressed_chunks: Mutex<Vec<Vec<u8>>> = Mutex::new(Vec::with_capacity(chunks.len()));

    pool.install(|| {
        chunks.into_par_iter().for_each(|chunk| {
            let compressed_chunk: Vec<u8> = unsafe {compress_chunk(chunk) };
            let mut chunks_guard: MutexGuard<Vec<Vec<u8>>> = compressed_chunks.lock().expect("Could not acquire compressed chunks mutex");
            chunks_guard.push(compressed_chunk);
        });
    });
    
    // combine chunks
    let compressed_chunks: Vec<Vec<u8>> = compressed_chunks.into_inner()
        .map_err(|e| format!("Could not acquire compressed chunks mutex: {e}"))?;
    Ok(compressed_chunks.concat())
}


fn optimal_thread_count(data_size: usize) -> usize {
    let available_threads: usize = rayon::current_num_threads();
    let min_chunk_size: usize = 1024 * 1024 * 10;   // 10MB min chunk
    let max_threads: usize = (available_threads as f32 * 0.75).ceil() as usize;     // use max 75% of available threads
    let thread_based_on_size: usize = std::cmp::max(1, data_size / min_chunk_size);
    std::cmp::min(max_threads, thread_based_on_size)
}

