use crate::prelude::*;

pub fn format_bytes(bytes: usize) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1000.0 && unit_idx < UNITS.len() - 1 {
        size /= 1000.0;
        unit_idx += 1;
    }

    format!("{:.1} {}", size, UNITS[unit_idx])
}

pub fn hexdump(raw_data: &[u8], range: impl std::ops::RangeBounds<usize>) -> Result<String> {
    use std::fmt::Write;
    use std::ops::Bound::*;

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
        bail!("Range out of bounds: {}..{} for length {}", start, end, len);
    }
    if start > end {
        bail!("Invalid range: start {} > end {}", start, end);
    }

    let slice = &raw_data[start..end];
    if slice.is_empty() {
        return Ok(String::new());
    }

    let mut string = String::with_capacity(slice.len() * 3 - 1);
    for (i, &byte) in slice.iter().enumerate() {
        if i > 0 {
            string.push(' ');
        }
        write!(&mut string, "{byte:02X}").unwrap();
    }

    Ok(string)
}

pub fn typename<T>() -> String {
    tynm::type_name::<T>()
}
