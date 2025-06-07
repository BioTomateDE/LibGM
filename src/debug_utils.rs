#[macro_export]
macro_rules! trace_build {
    ($label:expr, $expr:expr) => {{
        use crate::debug_utils::DurationExt;
        let _start = ::cpu_time::ProcessTime::now();
        let result = $expr;
        ::log::trace!("Building chunk '{}' took {}", $label, _start.elapsed().ms());
        result
    }};
}

#[macro_export]
macro_rules! trace_parse {
    ($label:expr, $expr:expr) => {{
        use crate::debug_utils::DurationExt;
        let _start = ::cpu_time::ProcessTime::now();
        let result = $expr;
        ::log::trace!("Parsing chunk '{}' took {}", $label, _start.elapsed().ms());
        result
    }};
}


pub trait DurationExt {
    fn ms(&self) -> String;
}

impl DurationExt for std::time::Duration {
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

    // Trim trailing `.0` for whole numbers
    if size.fract() == 0.0 {
        format!("{} {}", size as u64, UNITS[unit_idx])
    } else {
        format!("{:.1} {}", size, UNITS[unit_idx])
    }
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

