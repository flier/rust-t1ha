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
mod bits;
mod nightly;
mod t1ha0;

pub use t1ha0::{t1ha0_32be, t1ha0_32le};

#[cfg(target_endian = "little")]
pub use t1ha0::t1ha0_32le as t1ha0;

#[cfg(target_endian = "big")]
pub use t1ha0::t1ha0_32be as t1ha0;

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx"
))]
mod t1ha0_avx;

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx"
))]
pub use t1ha0_avx::t1ha0_ia32aes_avx;

#[cfg(test)]
mod selfcheck;
