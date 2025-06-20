use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::time::{Duration, Instant};
use cpu_time::ProcessTime;

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
        use crate::debug_utils::DurationExt;
        use ::colored::Colorize;
        let real: Duration = self.real_time.elapsed();
        let cpu: Duration = self.cpu_time.elapsed();
        write!(f, "{} {} {}{}", real.ms().bright_magenta(), "(cpu:".dimmed(), cpu.ms().magenta(), ")".dimmed())
    }
}


#[macro_export]
macro_rules! bench_export {
    ($label:expr, $expr:expr) => {{
        let _stopwatch = crate::debug_utils::Stopwatch::start();
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

/// how to use
/// - import crate once_cell
/// ```
/// static RUNTIME_STATS: Lazy<Mutex<RuntimeData>> = Lazy::new(|| 
///     Mutex::new(HashMap::new())
/// );
/// ...
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


/// to understand these seemingly meaningless functions, check out [PGO](https://doc.rust-lang.org/rustc/profile-guided-optimization.html)

#[inline(always)]
#[cold]
pub const fn cold() {}

#[inline(always)]
#[allow(unused)]
pub const fn likely(b: bool) -> bool {
    if !b {
        cold();
    }
    b
}

#[inline(always)]
pub const fn unlikely(b: bool) -> bool {
    if b {
        cold();
    }
    b
}


pub fn typename<T>() -> String {
    let string: &str = std::any::type_name::<T>();
    string.rsplit_once("::").map(|(_, i)| i).unwrap_or(string).to_string()
}

