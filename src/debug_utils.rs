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
macro_rules! bench_build {
    ($label:expr, $expr:expr) => {{
        let _stopwatch = crate::debug_utils::Stopwatch::start();
        let _result = $expr;
        ::log::trace!("Building chunk '{}' took {}", $label, _stopwatch);
        _result
    }};
}

#[macro_export]
macro_rules! bench_parse {
    ($label:expr, $expr:expr) => {{
        let _stopwatch = crate::debug_utils::Stopwatch::start();
        let _result = $expr;
        ::log::trace!("Parsing chunk '{}' took {}", $label, _stopwatch);
        _result
    }};
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

