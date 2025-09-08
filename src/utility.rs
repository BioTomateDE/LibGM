use cpu_time::ProcessTime;
use num_enum::TryFromPrimitive;
use std::collections::HashMap;
use std::fmt::{Display, Formatter, UpperHex};
use std::path::Path;
use std::time::{Duration, Instant};

pub struct Stopwatch {
    cpu_time: ProcessTime,
    real_time: Instant,
}
impl Stopwatch {
    pub fn start() -> Self {
        Self {
            cpu_time: ProcessTime::now(),
            real_time: Instant::now(),
        }
    }
    pub fn elapsed_real(&self) -> Duration {
        self.real_time.elapsed()
    }
    pub fn elapsed_cpu(&self) -> Duration {
        self.cpu_time.elapsed()
    }
}
impl Display for Stopwatch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use crate::utility::DurationExt;
        use ::colored::Colorize;
        let real: Duration = self.elapsed_real();
        let cpu: Duration = self.elapsed_cpu();
        write!(f, "{} {} {}{}", real.ms().bright_magenta(), "(cpu:".dimmed(), cpu.ms().magenta(), ")".dimmed())
    }
}


#[macro_export]
macro_rules! bench_export {
    ($label:expr, $expr:expr) => {{
        let _stopwatch = crate::utility::Stopwatch::start();
        let _result = $expr;
        ::log::trace!("Exporting {} took {}", $label, _stopwatch);
        _result
    }};
}


pub trait DurationExt {
    fn ms(&self) -> String;
}
impl DurationExt for Duration {
    fn ms(&self) -> String {
        format!("{:.2} ms", self.as_secs_f64() * 1000.0)
    }
}



pub(crate) type RuntimeData = HashMap<&'static str, (Duration, usize)>; // (total_time, call_count)

/// ===== How to use =====
/// - Add crate once_cell
/// ```ignore
/// static RUNTIME_STATS: Lazy<Mutex<RuntimeData>> = Lazy::new(|| 
///     Mutex::new(HashMap::new())
/// );
/// // ...
/// track_runtime!("my_function_name", {
///    // logic here
/// });
/// print_runtime_stats!();
/// ```
#[macro_export]
macro_rules! track_runtime {
    ($fn_name:expr, $code:block) => {{
        use ::std::time::{Instant, Duration};
        let start = Instant::now();
        let result = $code;
        let elapsed = start.elapsed();

        let mut stats = RUNTIME_STATS.lock().unwrap();
        let entry = stats.entry($fn_name).or_insert((Duration::ZERO, 0));
        entry.0 += elapsed;
        entry.1 += 1;

        result
    }};
}
#[macro_export]
macro_rules! print_runtime_stats {
    () => {{
       let stats = RUNTIME_STATS.lock().unwrap();
        println!("{:<20} | {:>12} | {:>10} | {:>12}", "Function", "Calls", "Total Time", "Avg/Call");
        println!("{}", "-".repeat(70));
        for (name, (total, calls)) in stats.iter() {
            let avg = *total / (*calls as u32).max(1);
            println!("{:<20} | {:>12} | {:>10.3?} | {:>10.3?}", name, calls, total, avg);
        }
    }}
}


pub fn format_bytes(bytes: usize) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    format!("{:.1} {}", size, UNITS[unit_idx])
}



pub fn typename<T>() -> String {
    tynm::type_name::<T>()
}

pub fn typename_val<T>(_: &T) -> String {
    tynm::type_name::<T>()
}


pub fn vec_with_capacity<T>(count: usize) -> Result<Vec<T>, String> {
    const FAILSAFE_SIZE: usize = 1_000_000;   // 1 Megabyte
    let implied_size = size_of::<T>() * count;
    if implied_size > FAILSAFE_SIZE {
        return Err(format!(
            "Failsafe triggered while initializing list of {}: \
            Element count {} implies a total data size of {} which is larger than the failsafe size of {}",
            typename::<T>(), count, format_bytes(implied_size), format_bytes(FAILSAFE_SIZE),
        ))
    }
    Ok(Vec::with_capacity(count))
}

pub fn hashmap_with_capacity<K, V>(count: usize) -> Result<HashMap<K, V>, String> {
    const FAILSAFE_SIZE: usize = 100_000;   // 100 KB
    let implied_size = size_of::<K>() * size_of::<V>() * count;
    if implied_size > FAILSAFE_SIZE {
        return Err(format!(
            "Failsafe triggered while initializing HashMap of <{}, {}>: \
            Element count {} implies a total data size of {} which is larger than the failsafe size of {}",
            typename::<K>(), typename::<V>(), count, format_bytes(implied_size), format_bytes(FAILSAFE_SIZE),
        ))
    }
    Ok(HashMap::with_capacity(count))
}


/// most readable rust function:
pub fn num_enum_from<I, N>(value: I) -> Result<N, String>
where
    I: Display + UpperHex + Copy,
    N: TryFromPrimitive + TryFrom<I>,
{
    match value.try_into() {
        // raw match statements for easy debugger breakpoints
        Ok(val) => Ok(val),
        Err(_) => Err(format!(
            "Invalid {0} {1} (0x{1:0width$X})",
            typename::<N>(),
            value,
            width = size_of::<I>() * 2,
        )),
    }
}


pub fn filename_to_str(path: &Path) -> String {
    path.file_name()
        .and_then(|os_str| os_str.to_str())
        .unwrap_or("<invalid filename>")
        .to_string()
}

