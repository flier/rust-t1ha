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
pub const PRIME_0: u64 = 0xEC99_BF0D_8372_CAAB;
pub const PRIME_1: u64 = 0x8243_4FE9_0EDC_EF39;
pub const PRIME_2: u64 = 0xD4F0_6DB9_9D67_BE4B;
pub const PRIME_3: u64 = 0xBD9C_ACC2_2C6E_9571;
pub const PRIME_4: u64 = 0x9C06_FAF4_D023_E3AB;
pub const PRIME_5: u64 = 0xC060_724A_8424_F345;
pub const PRIME_6: u64 = 0xCB5A_F53A_E3AA_AC31;

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
        let shift =
            ((mem::size_of::<T>() as isize - tail) & (mem::size_of::<T>() as isize - 1)) << 3;

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
        let shift =
            ((mem::size_of::<T>() as isize - tail) & (mem::size_of::<T>() as isize - 1)) << 3;

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
        let offset = (mem::size_of::<T>() as isize - tail) & (mem::size_of::<T>() as isize - 1);
        let shift = offset << 3;

        if likely(can_read_underside(p, mem::size_of::<T>())) {
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
        let offset = (mem::size_of::<T>() as isize - tail) & (mem::size_of::<T>() as isize - 1);
        let shift = offset << 3;

        if likely(can_read_underside(p, mem::size_of::<T>())) {
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

#[inline(always)]
pub fn rot64(v: u64, n: u32) -> u64 {
    v.rotate_right(n)
}

#[inline(always)]
pub fn final32(a: u32, b: u32) -> u64 {
    let mut l = u64::from(b ^ rot32(a, 13)) | u64::from(a) << 32;
    l = l.wrapping_mul(PRIME_0);
    l ^= l.wrapping_shr(41);
    l = l.wrapping_mul(PRIME_4);
    l ^= l.wrapping_shr(47);
    l = l.wrapping_mul(PRIME_6);
    l
}

#[inline(always)]
pub fn final64(a: u64, b: u64) -> u64 {
    let x = a.wrapping_add(rot64(b, 41)).wrapping_mul(PRIME_0);
    let y = rot64(a, 23).wrapping_add(b).wrapping_mul(PRIME_6);
    mux64(x ^ y, PRIME_5)
}

/// xor-mul-xor mixer
#[inline(always)]
pub fn mix64(v: u64, p: u64) -> u64 {
    let v = v.wrapping_mul(p);
    v ^ rot64(v, 41)
}

/// xor high and low parts of full 128-bit product
#[inline(always)]
pub fn mux64(v: u64, prime: u64) -> u64 {
    let mut h = 0;
    let l = mul_64x64_128(v, prime, &mut h);
    l ^ h
}

#[inline(always)]
pub fn mixup32(a: &mut u32, b: &mut u32, v: u32, prime: u32) {
    let l = mul_32x32_64(b.wrapping_add(v), prime);
    *a ^= l as u32;
    *b = b.wrapping_add((l >> 32) as u32);
}

#[inline(always)]
pub fn mixup64(a: &mut u64, b: &mut u64, v: u64, prime: u64) {
    let mut h = 0;
    *a ^= mul_64x64_128(b.wrapping_add(v), prime, &mut h);
    *b = b.wrapping_add(h);
}

#[inline(always)]
fn mul_32x32_64(a: u32, b: u32) -> u64 {
    u64::from(a).wrapping_mul(u64::from(b))
}

#[inline(always)]
fn mul_64x64_128(a: u64, b: u64, h: &mut u64) -> u64 {
    let r = u128::from(a).wrapping_mul(u128::from(b));
    *h = (r >> 64) as u64;
    r as u64
}
