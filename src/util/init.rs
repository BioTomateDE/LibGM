use crate::prelude::*;
use crate::util::fmt::{format_bytes, typename};
use num_enum::TryFromPrimitive;
use std::collections::HashMap;
use std::fmt::{Display, UpperHex};

pub fn vec_with_capacity<T>(count: u32) -> Result<Vec<T>> {
    const FAILSAFE_SIZE: usize = 10_000_000; // 10MB
    let count = count as usize;

    let implied_size = size_of::<T>() * count;
    if implied_size > FAILSAFE_SIZE {
        bail!(
            "{} count {} implies data size {} which exceeeds failsafe size {}",
            typename::<T>(),
            count,
            format_bytes(implied_size),
            format_bytes(FAILSAFE_SIZE),
        );
    }
    Ok(Vec::with_capacity(count))
}

pub fn hashmap_with_capacity<K, V>(count: u32) -> Result<HashMap<K, V>> {
    const FAILSAFE_SIZE: usize = 100_000; // 100 KB
    let count = count as usize;

    let entry_size = size_of::<(K, V)>() + size_of::<usize>() * 3;
    let estimated_size = entry_size * count;

    if estimated_size > FAILSAFE_SIZE {
        bail!(
            "HashMap<{}, {}> with capacity {} would use ~{} which exceeds failsafe size {}",
            std::any::type_name::<K>(),
            std::any::type_name::<V>(),
            count,
            format_bytes(estimated_size),
            format_bytes(FAILSAFE_SIZE)
        );
    }

    Ok(HashMap::with_capacity(count))
}

/// Most readable Rust Function:
pub fn num_enum_from<I, N>(value: I) -> Result<N>
where
    I: Display + UpperHex + Copy,
    N: TryFromPrimitive + TryFrom<I>,
{
    match value.try_into() {
        // Raw match statements for easy debugger breakpoints
        Ok(val) => Ok(val),
        Err(_) => bail!(
            "Invalid {0} {1} (0x{1:0width$X})",
            typename::<N>(),
            value,
            width = size_of::<I>() * 2,
        ),
    }
}
