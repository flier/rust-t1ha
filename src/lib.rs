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
mod t1ha1;
mod t1ha2;

pub use t1ha0::{t1ha0_32be, t1ha0_32le};
pub use t1ha1::{t1ha1_32be, t1ha1_32le};
pub use t1ha2::{t1ha2_atonce, t1ha2_atonce128, T1ha2Hasher};

#[cfg(target_endian = "little")]
pub use t1ha0::t1ha0_32le as t1ha0;

#[cfg(target_endian = "big")]
pub use t1ha0::t1ha0_32be as t1ha0;

#[cfg(target_endian = "little")]
pub use t1ha1::t1ha1_32le as t1ha1;

#[cfg(target_endian = "big")]
pub use t1ha1::t1ha1_32be as t1ha1;

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    any(
        target_feature = "aes",
        target_feature = "avx",
        target_feature = "avx2"
    ),
))]
mod t1ha0_aes;

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    all(
        target_feature = "aes",
        not(any(target_feature = "avx", target_feature = "avx2"))
    )
))]
pub use t1ha0_aes::t1ha0_ia32aes as t1ha0_ia32aes_noavx;

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx"
))]
pub use t1ha0_aes::t1ha0_ia32aes as t1ha0_ia32aes_avx;

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
pub use t1ha0_aes::t1ha0_ia32aes_avx2;

#[cfg(test)]
mod selfcheck;
