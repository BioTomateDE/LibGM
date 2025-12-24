use std::ops::RangeBounds;

use crate::prelude::*;

pub fn format_bytes(bytes: usize) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];

    #[allow(clippy::cast_precision_loss)]
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1000.0 && unit_idx < UNITS.len() - 1 {
        size /= 1000.0;
        unit_idx += 1;
    }

    format!("{:.1} {}", size, UNITS[unit_idx])
}

pub fn hexdump(raw_data: &[u8]) -> String {
    if raw_data.is_empty() {
        return String::new();
    }

    let mut buffer = String::with_capacity(raw_data.len() * 3);
    for byte in raw_data {
        use std::fmt::Write as _;
        write!(&mut buffer, "{byte:02X} ").unwrap();
    }

    // Pop last space character.
    buffer.pop();

    buffer
}

pub fn hexdump_range(raw_data: &[u8], range: impl RangeBounds<usize>) -> Result<String> {
    use std::ops::Bound::{Excluded, Included, Unbounded};

    let len = raw_data.len();
    let start = match range.start_bound() {
        Included(&n) => n,
        Excluded(&n) => n + 1,
        Unbounded => 0,
    };
    let end = match range.end_bound() {
        Included(&n) => n + 1,
        Excluded(&n) => n,
        Unbounded => len,
    };

    if start > len || end > len {
        bail!("Range out of bounds: {start}..{end} for length {len}");
    }
    if start > end {
        bail!("Invalid range: start {start} > end {end}");
    }

    let slice: &[u8] = &raw_data[start..end];
    Ok(hexdump(slice))
}

/// This function should only ever be called when an error has already occurred.
///
/// It basically just gets the typename in a slightly more readable manner.
pub fn typename<T>() -> &'static str {
    let ty = std::any::type_name::<T>();
    if let Some(index) = ty.find("GM") {
        return &ty[index..];
    }
    ty.strip_prefix("libgm::gamemaker::elements::").unwrap_or(ty)
}
