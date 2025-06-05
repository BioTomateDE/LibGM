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

pub unsafe fn compress_chunk(input: &[u8]) -> Result<Vec<u8>, String> {
    let block_size: i32 = 9;     // max compression (900k blocks)
    let mut output_len: u32 = (input.len() as f32 * 1.01) as u32 + 600;   // bzip2's max formula

    let mut output = Vec::with_capacity(output_len as usize);
    let ret = BZ2_bzBuffToBuffCompress(
        output.as_mut_ptr() as *mut c_char,
        &mut output_len,
        input.as_ptr() as *const c_char,
        input.len() as u32,
        block_size,
        0, // verbosity
        30, // workFactor
    );

    if ret != 0 {
        return Err(format!("BZip2 compression failed with error code {ret}"))
    }
    output.set_len(output_len as usize);
    Ok(output)
}

pub fn compress_parallel(input: &[u8], chunk_size: usize) -> Result<Vec<u8>, String> {
    let num_threads: usize = optimal_thread_count(input.len());
    let pool: ThreadPool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .map_err(|e| format!("Could not build rayon ThreadPool: {e}"))?;

    let chunks: Vec<&[u8]> = input.chunks(chunk_size).collect::<Vec<_>>();
    let compressed_chunks: Arc<Mutex<Vec<Vec<u8>>>> = Arc::new(Mutex::new(Vec::with_capacity(chunks.len())));
    let error_flag: Arc<Mutex<Option<Arc<String>>>> = Arc::new(Mutex::new(None));

    pool.install(|| {
        chunks.into_par_iter().for_each(|chunk| {
            fn set_err(error_flag: &Arc<Mutex<Option<Arc<String>>>>, e: String) {
                *error_flag.lock().expect("Could not acquire error flag mutex while compressing bzip") = Some(Arc::new(e));
            }

            if !error_flag.lock().is_ok_and(|i| i.is_none()) {
                return;   // early return if another thread failed or panicked
            }
            match unsafe {compress_chunk(chunk) } {
                Ok(compressed) => {
                    let mut chunks_guard: MutexGuard<Vec<Vec<u8>>> = match compressed_chunks.lock() {
                        Ok(guard) => guard,
                        Err(e) => {
                            set_err(&error_flag, format!("Could not acquire compressed chunks mutex: {e}"));
                            return
                        },
                    };
                    chunks_guard.push(compressed);
                }
                Err(e) => set_err(&error_flag, e)
            }
        });
    });

    // disgusting error handling (just to avoid unwrap)
    let error_flag: Option<Arc<String>> = Arc::try_unwrap(error_flag)
        .map_err(|_| "Could not unwrap error flag arc".to_string())?
        .into_inner()
        .map_err(|e| format!("Could not acquire error flag mutex: {e}"))?;
    
    if let Some(error_arc) = error_flag {
        let error_str: String = Arc::try_unwrap(error_arc)
            .map_err(|_| "Could not unwrap error flag arc".to_string())?;
        return Err(error_str);
    }
    
    // combine chunks
    let compressed_chunks: Vec<Vec<u8>> = Arc::try_unwrap(compressed_chunks)
        .map_err(|_| "Could not unwrap compressed chunks arc".to_string())?
        .into_inner()
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

