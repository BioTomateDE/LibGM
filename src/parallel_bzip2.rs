use std::ffi::{c_char, c_int};
use std::mem::MaybeUninit;
use rayon::prelude::*;
use std::sync::{Mutex, MutexGuard};
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

unsafe fn compress_chunk(input: &[u8]) -> Result<Vec<u8>, &'static str> {
    // preallocate maximum possible output size (1% + 600 bytes overhead)
    let mut output_len: u32 = (input.len() as f32 * 1.01) as u32 + 600;
    let output: Vec<u8> = Vec::with_capacity(output_len as usize);

    // use MaybeUninit to avoid zeroing memory
    let mut output: MaybeUninit<Vec<u8>> = MaybeUninit::new(output);
    let output_ptr: *mut Vec<u8> = output.as_mut_ptr();

    // perform compression directly into the vector's buffer
    let ret = BZ2_bzBuffToBuffCompress(
        (*output_ptr).as_mut_ptr() as *mut c_char,
        &mut output_len,
        input.as_ptr() as *const c_char,
        input.len() as u32,
        9,      // 900k block size (max compression)
        0,         // no verbosity
        30,      // default work factor
    );

    if ret != 0 {
        return Err(match ret {
            1 => "BZ_CONFIG_ERROR",
            2 => "BZ_PARAM_ERROR",
            3 => "BZ_MEM_ERROR",
            4 => "BZ_OUTBUFF_FULL",
            _ => "BZ_UNKNOWN_ERROR",
        });
    }

    // SAFETY: capacity is sufficient
    let mut output: Vec<u8> = output.assume_init();
    output.set_len(output_len as usize);

    // shrink to fit if it was over allocated significantly
    if output.capacity() > output.len() * 2 {
        output.shrink_to_fit();
    }

    Ok(output)
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
            let compressed_chunk: Vec<u8> = unsafe {
                match compress_chunk(chunk) {
                    Ok(c) => c,
                    Err(e) => panic!("Error while bzip2 compressing chunk: {e}")
                }
            };
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
    let thread_based_on_size: usize = std::cmp::max(1, data_size / min_chunk_size);
    std::cmp::min(available_threads, thread_based_on_size)
}

