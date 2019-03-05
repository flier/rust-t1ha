use core::marker::PhantomData;
use core::mem;
use core::ptr;

use num_traits::{PrimInt, WrappingShr, Zero};

use crate::nightly::likely;

const PAGESIZE: usize = 4096;

pub fn aligned_to<T, P>(p: *const P) -> bool {
    (p as usize) % mem::size_of::<T>() == 0
}

// 'magic' primes
pub const PRIME_0: u64 = 0xEC99BF0D8372CAAB;
pub const PRIME_1: u64 = 0x82434FE90EDCEF39;
pub const PRIME_2: u64 = 0xD4F06DB99D67BE4B;
pub const PRIME_3: u64 = 0xBD9CACC22C6E9571;
pub const PRIME_4: u64 = 0x9C06FAF4D023E3AB;
pub const PRIME_5: u64 = 0xC060724A8424F345;
pub const PRIME_6: u64 = 0xCB5AF53AE3AAAC31;

#[inline(always)]
fn can_read_underside<T>(ptr: *const T, size: usize) -> bool {
    ((PAGESIZE - size) & (ptr as usize)) != 0
}

pub trait MemoryModel {
    type Item;

    unsafe fn fetch<P>(p: *const P) -> Self::Item;

    unsafe fn tail<P>(p: *const P, len: isize) -> Self::Item;
}

pub struct LittenEndianAligned<T>(PhantomData<T>);
pub struct BigEndianAligned<T>(PhantomData<T>);
pub struct LittenEndianUnaligned<T>(PhantomData<T>);
pub struct BigEndianUnaligned<T>(PhantomData<T>);

impl<T> MemoryModel for LittenEndianAligned<T>
where
    T: PrimInt + Zero + WrappingShr,
{
    type Item = T;

    #[inline(always)]
    unsafe fn fetch<P>(p: *const P) -> Self::Item {
        debug_assert!(aligned_to::<Self::Item, _>(p));

        ptr::read(p as *const Self::Item).to_le()
    }

    #[inline(always)]
    unsafe fn tail<P>(p: *const P, tail: isize) -> Self::Item {
        let p = p as *const u8;
        let shift = ((4 - tail) & 3) << 3;

        Self::fetch(p) & ((!T::zero()).wrapping_shr(shift as u32))
    }
}

impl<T> MemoryModel for BigEndianAligned<T>
where
    T: PrimInt + Zero + WrappingShr,
{
    type Item = T;

    #[inline(always)]
    unsafe fn fetch<P>(p: *const P) -> Self::Item {
        debug_assert!(aligned_to::<Self::Item, _>(p));

        ptr::read(p as *const Self::Item).to_be()
    }

    #[inline(always)]
    unsafe fn tail<P>(p: *const P, tail: isize) -> Self::Item {
        let p = p as *const u8;
        let shift = ((4 - tail) & 3) << 3;

        Self::fetch(p).wrapping_shr(shift as u32)
    }
}

impl<T> MemoryModel for LittenEndianUnaligned<T>
where
    T: PrimInt + Zero + WrappingShr,
{
    type Item = T;

    #[inline(always)]
    unsafe fn fetch<P>(p: *const P) -> Self::Item {
        ptr::read_unaligned(p as *const Self::Item).to_le()
    }

    #[inline(always)]
    unsafe fn tail<P>(p: *const P, tail: isize) -> Self::Item {
        let p = p as *const u8;
        let offset = (4 - tail) & 3;
        let shift = offset << 3;

        if likely(can_read_underside(p, 4)) {
            Self::fetch(p.sub(offset as usize)).wrapping_shr(shift as u32)
        } else {
            Self::fetch(p) & ((!T::zero()).wrapping_shr(shift as u32))
        }
    }
}

impl<T> MemoryModel for BigEndianUnaligned<T>
where
    T: PrimInt + Zero + WrappingShr,
{
    type Item = T;

    #[inline(always)]
    unsafe fn fetch<P>(p: *const P) -> Self::Item {
        ptr::read_unaligned(p as *const Self::Item).to_be()
    }

    #[inline(always)]
    unsafe fn tail<P>(p: *const P, tail: isize) -> Self::Item {
        let p = p as *const u8;
        let offset = (4 - tail) & 3;
        let shift = offset << 3;

        if likely(can_read_underside(p, 4)) {
            Self::fetch(p.sub(offset as usize)) & ((!T::zero()).wrapping_shr(shift as u32))
        } else {
            Self::fetch(p).wrapping_shr(shift as u32)
        }
    }
}

#[inline(always)]
pub fn rot32(v: u32, n: u32) -> u32 {
    v.rotate_right(n)
}
