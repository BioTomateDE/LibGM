// SPDX-License-Identifier: GPL-3.0-only
use std::collections::HashMap;

use crate::prelude::*;
use crate::util::fmt::format_bytes;
use crate::util::fmt::typename;

pub fn vec_with_capacity<T>(count: u32) -> Result<Vec<T>> {
    const FAILSAFE_SIZE: usize = 10_000_000; // 10MB
    let count = count as usize;

    let implied_size = size_of::<T>().saturating_mul(count);
    if implied_size > FAILSAFE_SIZE {
        bail!(
            "{} count {} implies data size {} which exceeds failsafe size {}",
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
    let estimated_size = entry_size.saturating_mul(count);

    if estimated_size > FAILSAFE_SIZE {
        bail!(
            "HashMap<{}, {}> with capacity {} would use ~{} which exceeds failsafe size {}",
            typename::<K>(),
            typename::<V>(),
            count,
            format_bytes(estimated_size),
            format_bytes(FAILSAFE_SIZE),
        );
    }

    Ok(HashMap::with_capacity(count))
}
