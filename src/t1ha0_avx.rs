#[cfg(target_arch = "x86")]
use core::arch::x86::*;

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use crate::{bits::*, nightly::*};

pub fn t1ha0_ia32aes_avx(data: &[u8], seed: u64) -> u64 {
    let mut len = data.len();
    let mut a = seed;
    let mut b = len as u64;
    let mut p = data.as_ptr();

    unsafe {
        if unlikely(len > 32) {
            let mut x = _mm_set_epi64x(a as i64, b as i64);
            let mut y = _mm_aesenc_si128(x, _mm_set_epi64x(PRIME_5 as i64, PRIME_6 as i64));
            let mut v = p as *const __m128i;
            let detent = p as usize + len - 127;

            while likely((v as usize) < detent) {
                let v0 = _mm_loadu_si128(v.offset(0));
                let v1 = _mm_loadu_si128(v.offset(1));
                let v2 = _mm_loadu_si128(v.offset(2));
                let v3 = _mm_loadu_si128(v.offset(3));
                let v4 = _mm_loadu_si128(v.offset(4));
                let v5 = _mm_loadu_si128(v.offset(5));
                let v6 = _mm_loadu_si128(v.offset(6));
                let v7 = _mm_loadu_si128(v.offset(7));
                v = v.add(8);
                prefetch(v);

                let v0y = _mm_aesenc_si128(v0, y);
                let v2x6 = _mm_aesenc_si128(v2, _mm_xor_si128(x, v6));
                let v45_67 = _mm_xor_si128(_mm_aesenc_si128(v4, v5), _mm_add_epi64(v6, v7));

                let v0y7_1 = _mm_aesdec_si128(_mm_sub_epi64(v7, v0y), v1);
                let v2x6_3 = _mm_aesenc_si128(v2x6, v3);

                x = _mm_aesenc_si128(v45_67, _mm_add_epi64(x, y));
                y = _mm_aesenc_si128(v2x6_3, _mm_xor_si128(v0y7_1, v5));
            }

            if (len & 64) != 0 {
                let v0y = _mm_add_epi64(y, _mm_loadu_si128(v.offset(0)));
                let v1x = _mm_sub_epi64(x, _mm_loadu_si128(v.offset(1)));
                x = _mm_aesdec_si128(x, v0y);
                y = _mm_aesdec_si128(y, v1x);

                let v2y = _mm_add_epi64(y, _mm_loadu_si128(v.offset(2)));
                let v3x = _mm_sub_epi64(x, _mm_loadu_si128(v.offset(3)));
                x = _mm_aesdec_si128(x, v2y);
                y = _mm_aesdec_si128(y, v3x);

                v = v.add(4)
            }

            if (len & 32) != 0 {
                let v0y = _mm_add_epi64(y, _mm_loadu_si128(v.offset(0)));
                let v1x = _mm_sub_epi64(x, _mm_loadu_si128(v.offset(1)));
                x = _mm_aesdec_si128(x, v0y);
                y = _mm_aesdec_si128(y, v1x);

                v = v.add(2)
            }

            if (len & 16) != 0 {
                y = _mm_add_epi64(x, y);
                x = _mm_aesdec_si128(x, _mm_loadu_si128(v));
                v = v.add(1)
            }

            x = _mm_add_epi64(_mm_aesdec_si128(x, _mm_aesenc_si128(y, x)), y);

            let (lo, hi) = extract_i64(x);

            a = lo as u64;
            b = hi as u64;

            mm_empty();

            p = v as *const _;
            len &= 15;
        }

        let v = p as *const u64;

        match len {
            32 | 31 | 30 | 29 | 28 | 27 | 26 | 25 => {
                mixup64(
                    &mut a,
                    &mut b,
                    LittenEndianUnaligned::<u64>::fetch(v.offset(0)),
                    PRIME_4,
                );
                mixup64(
                    &mut b,
                    &mut a,
                    LittenEndianUnaligned::<u64>::fetch(v.offset(1)),
                    PRIME_3,
                );
                mixup64(
                    &mut a,
                    &mut b,
                    LittenEndianUnaligned::<u64>::fetch(v.offset(2)),
                    PRIME_2,
                );
                mixup64(
                    &mut b,
                    &mut a,
                    LittenEndianUnaligned::<u64>::tail(v.offset(3), len as isize),
                    PRIME_1,
                );
                final64(a, b)
            }
            24 | 23 | 22 | 21 | 20 | 19 | 18 | 17 => {
                mixup64(
                    &mut b,
                    &mut a,
                    LittenEndianUnaligned::<u64>::fetch(v.offset(0)),
                    PRIME_3,
                );
                mixup64(
                    &mut a,
                    &mut b,
                    LittenEndianUnaligned::<u64>::fetch(v.offset(1)),
                    PRIME_2,
                );
                mixup64(
                    &mut b,
                    &mut a,
                    LittenEndianUnaligned::<u64>::tail(v.offset(2), len as isize),
                    PRIME_1,
                );
                final64(a, b)
            }
            16 | 15 | 14 | 13 | 12 | 11 | 10 | 9 => {
                mixup64(
                    &mut a,
                    &mut b,
                    LittenEndianUnaligned::<u64>::fetch(v.offset(0)),
                    PRIME_2,
                );
                mixup64(
                    &mut b,
                    &mut a,
                    LittenEndianUnaligned::<u64>::tail(v.offset(1), len as isize),
                    PRIME_1,
                );
                final64(a, b)
            }
            8 | 7 | 6 | 5 | 4 | 3 | 2 | 1 => {
                mixup64(
                    &mut b,
                    &mut a,
                    LittenEndianUnaligned::<u64>::tail(v, len as isize),
                    PRIME_1,
                );
                final64(a, b)
            }
            0 => final64(a, b),
            _ => unreachable!(),
        }
    }
}

#[cfg(target_feature = "avx")]
unsafe fn mm_empty() {
    _mm256_zeroupper();
}

#[cfg(not(target_feature = "avx"))]
unsafe fn mm_empty() {
    _mm_empty();
}

#[cfg(all(
    target_arch = "x86_64",
    any(target_feature = "sse4.1", target_feature = "avx")
))]
unsafe fn extract_i64(x: __m128i) -> (i64, i64) {
    (_mm_extract_epi64(x, 0), _mm_extract_epi64(x, 1))
}

#[cfg(all(
    target_arch = "x86_64",
    not(any(target_feature = "sse4.1", target_feature = "avx"))
))]
unsafe fn extract_i64(x: __m128i) -> (i64, i64) {
    (
        _mm_cvtsi128_si64(x),
        _mm_cvtsi128_si64(_mm_unpackhi_epi64(x, x)),
    )
}

#[cfg(all(
    target_arch = "x86",
    any(target_feature = "sse4.1", target_feature = "avx")
))]
unsafe fn extract_i64(x: __m128i) -> (i64, i64) {
    (
        u32::from(_mm_extract_epi32(x, 0)) | u64::from(_mm_extract_epi32(x, 1)) << 32,
        u32::from(_mm_extract_epi32(x, 2)) | u64::from(_mm_extract_epi32(x, 3)) << 32,
    )
}

#[cfg(all(
    target_arch = "x86",
    not(any(target_feature = "sse4.1", target_feature = "avx"))
))]
unsafe fn extract_i64(x: __m128i) -> (i64, i64) {
    let lo =
        _mm_cvtsi128_si32(x) as u32 | u64::from(_mm_cvtsi128_si32(_mm_shuffle_epi32(x, 1))) << 32;
    let x = _mm_unpackhi_epi64(x, x);
    let hi =
        _mm_cvtsi128_si32(x) as u32 | u64::from(_mm_cvtsi128_si32(_mm_shuffle_epi32(x, 1))) << 32;

    (lo, hi)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selfcheck::selfcheck;

    const T1HA_REFVAL_IA32AES: [u64; 81] = [
        0,
        0x772C7311BE32FF42,
        0xB231AC660E5B23B5,
        0x71F6DF5DA3B4F532,
        0x555859635365F660,
        0xE98808F1CD39C626,
        0x2EB18FAF2163BB09,
        0x7B9DD892C8019C87,
        0xE2B1431C4DA4D15A,
        0x1984E718A5477F70,
        0x08DD17B266484F79,
        0x4C83A05D766AD550,
        0x92DCEBB131D1907D,
        0xD67BC6FC881B8549,
        0xF6A9886555FBF66B,
        0x6E31616D7F33E25E,
        0x36E31B7426E3049D,
        0x4F8E4FAF46A13F5F,
        0x03EB0CB3253F819F,
        0x636A7769905770D2,
        0x3ADF3781D16D1148,
        0x92D19CB1818BC9C2,
        0x283E68F4D459C533,
        0xFA83A8A88DECAA04,
        0x8C6F00368EAC538C,
        0x7B66B0CF3797B322,
        0x5131E122FDABA3FF,
        0x6E59FF515C08C7A9,
        0xBA2C5269B2C377B0,
        0xA9D24FD368FE8A2B,
        0x22DB13D32E33E891,
        0x7B97DFC804B876E5,
        0xC598BDFCD0E834F9,
        0xB256163D3687F5A7,
        0x66D7A73C6AEF50B3,
        0xBB34C6A4396695D2,
        0x7F46E1981C3256AD,
        0x4B25A9B217A6C5B4,
        0x7A0A6BCDD2321DA9,
        0x0A1F55E690A7B44E,
        0x8F451A91D7F05244,
        0x624D5D3C9B9800A7,
        0x09DDC2B6409DDC25,
        0x3E155765865622B6,
        0x96519FAC9511B381,
        0x512E58482FE4FBF0,
        0x1AB260EA7D54AE1C,
        0x67976F12CC28BBBD,
        0x0607B5B2E6250156,
        0x7E700BEA717AD36E,
        0x06A058D9D61CABB3,
        0x57DA5324A824972F,
        0x1193BA74DBEBF7E7,
        0xC18DC3140E7002D4,
        0x9F7CCC11DFA0EF17,
        0xC487D6C20666A13A,
        0xB67190E4B50EF0C8,
        0xA53DAA608DF0B9A5,
        0x7E13101DE87F9ED3,
        0x7F8955AE2F05088B,
        0x2DF7E5A097AD383F,
        0xF027683A21EA14B5,
        0x9BB8AEC3E3360942,
        0x92BE39B54967E7FE,
        0x978C6D332E7AFD27,
        0xED512FE96A4FAE81,
        0x9E1099B8140D7BA3,
        0xDFD5A5BE1E6FE9A6,
        0x1D82600E23B66DD4,
        0x3FA3C3B7EE7B52CE,
        0xEE84F7D2A655EF4C,
        0x2A4361EC769E3BEB,
        0x22E4B38916636702,
        0x0063096F5D39A115,
        0x6C51B24DAAFA5434,
        0xBAFB1DB1B411E344,
        0xFF529F161AE0C4B0,
        0x1290EAE3AC0A686F,
        0xA7B0D4585447D1BE,
        0xAED3D18CB6CCAD53,
        0xFC73D46F8B41BEC6,
    ];

    #[test]
    fn test_ia32aes_avx() {
        selfcheck(t1ha0_ia32aes_avx, &T1HA_REFVAL_IA32AES[..])
    }
}
