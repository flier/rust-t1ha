//!  t1ha = { Fast Positive Hash, aka "Позитивный Хэш" }
//!  by [Positive Technologies](https://www.ptsecurity.ru)
//!
//!  Briefly, it is a 64-bit Hash Function:
//!   1. Created for 64-bit little-endian platforms, in predominantly for x86_64,
//!      but portable and without penalties it can run on any 64-bit CPU.
//!   2. In most cases up to 15% faster than City64, xxHash, mum-hash, metro-hash
//!      and all others portable hash-functions (which do not use specific
//!      hardware tricks).
//!   3. Not suitable for cryptography.
//!
//!  The Future will Positive. Всё будет хорошо.
//!
//!  ACKNOWLEDGEMENT:
//!  The t1ha was originally developed by Leonid Yuriev (Леонид Юрьев)
//!  for The 1Hippeus project - zerocopy messaging in the spirit of Sparta!
#![no_std]

#[cfg(any(test, feature = "std"))]
extern crate std;
#[macro_use]
extern crate cfg_if;

use core::hash::{BuildHasherDefault, Hasher};

mod bits;
mod nightly;
mod t1ha0;
mod t1ha1;
mod t1ha2;

pub use t1ha0::{t1ha0_32be, t1ha0_32le};
pub use t1ha1::{t1ha1_be, t1ha1_le};
pub use t1ha2::{t1ha2_atonce, t1ha2_atonce128, T1ha2Hasher};

cfg_if! {
    if #[cfg(target_endian = "little")] {
        pub use t1ha0::t1ha0_32le as t1ha0_32;
        pub use t1ha1::t1ha1_le as t1ha1;
    } else {
        pub use t1ha0::t1ha0_32be as t1ha0_32;
        pub use t1ha1::t1ha1_be as t1ha1;
    }
}

cfg_if! {
    if #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] {
        cfg_if! {
            if #[cfg(any(target_feature = "aes", target_feature = "avx", target_feature = "avx2"))] {
                mod t1ha0_aes;
            }
        }
        cfg_if! {
            if #[cfg(target_feature = "aes")] {
                pub use t1ha0_aes::t1ha0_ia32aes as t1ha0_ia32aes_noavx;
            }
        }
        cfg_if! {
            if #[cfg(target_feature = "avx")] {
                pub use t1ha0_aes::t1ha0_ia32aes as t1ha0_ia32aes_avx;
            }
        }
        cfg_if! {
            if #[cfg(target_feature = "avx2")] {
                pub use t1ha0_aes::t1ha0_ia32aes_avx2;
            }
        }
    }
}

/// An implementation of the `t1ha` hash function.
///
/// See the [crate documentation](index.html) for more details.
#[derive(Clone, Copy, Debug, Default)]
pub struct T1haHasher(u64);

impl T1haHasher {
    /// Create a `t1ha` hasher starting with a state corresponding to the hash `key`.
    #[inline]
    pub fn with_seed(seed: u64) -> Self {
        Self(seed)
    }
}

impl Hasher for T1haHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        *self = T1haHasher(t1ha0(bytes, self.0));
    }
}

/// A builder for default `t1ha` hashers.
pub type T1haBuildHasher = BuildHasherDefault<T1haHasher>;

cfg_if! {
    if #[cfg(feature = "std")] {
        use std::collections::{HashMap, HashSet};

        /// A `HashMap` using a default `t1ha` hasher.
        pub type T1haHashMap<K, V> = HashMap<K, V, T1haBuildHasher>;

        /// A `HashSet` using a default `t1ha` hasher.
        pub type T1haHashSet<T> = HashSet<T, T1haBuildHasher>;
    }
}

/// `t1ha` library offers the t1ha0() function as the fastest for current CPU.
///
/// But actual CPU's features/capabilities and may be significantly different,
/// especially on x86 platform. Therefore, internally, t1ha0() may require
/// dynamic dispatching for choice best implementation.
pub fn t1ha0(data: &[u8], seed: u64) -> u64 {
    prefer::HASH(data, seed)
}

mod prefer {
    cfg_if! {
        if #[cfg(target_pointer_width = "64")] {
            pub use super::t1ha1 as t1ha0_fallback;
        } else {
            pub use super::t1ha0_32 as t1ha0_fallback;
        }
    }

    cfg_if! {
        if #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), feature = "runtime_select", feature = "std"))] {
            use std::is_x86_feature_detected;

            use lazy_static::lazy_static;

            lazy_static! {
                pub static ref HASH: fn(data: &[u8], seed: u64) -> u64 = t1ha0_resolve();
            }

            #[cfg(not(target_feature = "aes"))]
            pub use self::t1ha0_fallback as t1ha0_ia32aes_noavx;

            #[cfg(not(target_feature = "avx"))]
            pub use self::t1ha0_fallback as t1ha0_ia32aes_avx;

            #[cfg(not(target_feature = "avx2"))]
            pub use self::t1ha0_fallback as t1ha0_ia32aes_avx2;

            fn t1ha0_resolve() -> fn(&[u8], u64) -> u64 {
                if is_x86_feature_detected!("avx2") {
                    t1ha0_ia32aes_avx2
                } else if is_x86_feature_detected!("avx") {
                    t1ha0_ia32aes_avx
                } else if is_x86_feature_detected!("aes") {
                    t1ha0_ia32aes_noavx
                } else {
                    t1ha0_fallback
                }
            }
        } else {
            pub use self::t1ha0_fallback as HASH;
        }
    }
}

#[cfg(test)]
mod selfcheck;
