use std::path::Path;

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

pub fn filename_to_str(path: &Path) -> String {
    path.file_name()
        .and_then(|os_str| os_str.to_str())
        .unwrap_or("<invalid filename>")
        .to_string()
}

pub fn typename<T>() -> String {
    tynm::type_name::<T>()
}

pub fn typename_val<T>(_: &T) -> String {
    tynm::type_name::<T>()
}

