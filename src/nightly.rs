#[cfg(feature = "nightly")]
use core::intrinsics::{likely, prefetch_read_data, unlikely};

#[cfg(not(feature = "nightly"))]
#[inline(always)]
pub fn likely(b: bool) -> bool {
    b
}

#[cfg(not(feature = "nightly"))]
#[inline(always)]
pub fn unlikely(b: bool) -> bool {
    b
}

#[cfg(feature = "nightly")]
#[inline(always)]
pub fn prefetch<T>(data: *const T) {
    prefetch_read_data(data, 2) // locality (0) - no locality, to (3) - extremely local keep in cache.
}

#[cfg(not(feature = "nightly"))]
#[inline(always)]
pub fn prefetch<T>(_data: *const T) {}
