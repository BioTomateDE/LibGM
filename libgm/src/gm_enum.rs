// SPDX-License-Identifier: GPL-3.0-only
use core::fmt::Debug;
use core::hash::Hash;

use crate::prelude::*;
use crate::util::fmt::typename;

pub trait GMEnum: Copy + Eq + Hash + Debug {
    #[must_use]
    fn try_from_i32(integer: i32) -> Option<Self>;

    fn from_i32(integer: i32) -> Result<Self> {
        Self::try_from_i32(integer).ok_or_else(|| {
            err!(
                "Invalid {} enum value {} (0x{:016X})",
                typename::<Self>(),
                integer,
                integer,
            )
        })
    }

    #[must_use]
    fn as_i32(self) -> i32;
}

macro_rules! gm_enum {
    ($(#[$meta:meta])* $name:ident { $( $(#[$vmeta:meta])* $variant:ident = $int:literal, )+  } ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name {
            $( $(#[$vmeta])* $variant, )+
        }

        impl crate::gm_enum::GMEnum for $name {
            fn try_from_i32(integer: i32) -> Option<Self> {
                match integer {
                    $( $int => Some(Self::$variant), )+
                    _ => None,
                }
            }

            fn as_i32(self) -> i32 {
                match self {
                    $( Self::$variant => $int, )+
                }
            }
        }
    };
}

pub(crate) use gm_enum;
