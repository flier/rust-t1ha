#[cfg(feature = "nightly")]
use core::intrinsics::{likely, prefetch_read_data as prefetch, unlikely};

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

#[cfg(not(feature = "nightly"))]
#[inline(always)]
pub fn prefetch<T>(_data: *const T, _locality: i32) {}
