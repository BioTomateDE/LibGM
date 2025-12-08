#[inline(always)]
#[cold]
pub const fn cold_path() {}

#[inline]
#[must_use = "use `cold_path` if you don't want a bool parameter / return value"]
pub const fn unlikely(b: bool) -> bool {
    if b {
        cold_path();
    }
    b
}

#[allow(dead_code)] // remove this later if never used ig
#[inline]
#[must_use]
pub const fn likely(b: bool) -> bool {
    if !b {
        cold_path();
    }
    b
}
