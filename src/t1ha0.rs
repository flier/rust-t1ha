#![allow(clippy::cast_ptr_alignment)]

use crate::{bits::*, nightly::*};

// 32-bit 'magic' primes
const PRIME32_0: u32 = 0x92D7_8269;
const PRIME32_1: u32 = 0xCA9B_4735;
const PRIME32_2: u32 = 0xA4AB_A1C3;
const PRIME32_3: u32 = 0xF649_9843;
const PRIME32_4: u32 = 0x86F0_FD61;
const PRIME32_5: u32 = 0xCA2D_A6FB;
const PRIME32_6: u32 = 0xC4BB_3575;

/// The little-endian variant for 32-bit CPU.
#[cfg(feature = "unaligned_access")]
pub fn t1ha0_32le(data: &[u8], seed: u64) -> u64 {
    unsafe { t1h0_body::<LittenEndianUnaligned<u32>>(data, seed) }
}

/// The little-endian variant for 32-bit CPU.
#[cfg(not(feature = "unaligned_access"))]
pub fn t1ha0_32le(data: &[u8], seed: u64) -> u64 {
    if !aligned_to::<u32, _>(data.as_ptr()) {
        unsafe { t1h0_body::<LittenEndianUnaligned<u32>>(data, seed) }
    } else {
        unsafe { t1h0_body::<LittenEndianAligned<u32>>(data, seed) }
    }
}

/// The big-endian variant for 32-bit CPU.
#[cfg(feature = "unaligned_access")]
pub fn t1ha0_32be(data: &[u8], seed: u64) -> u64 {
    unsafe { t1h0_body::<BigEndianUnaligned<u32>>(data, seed) }
}

/// The big-endian variant for 32-bit CPU.
#[cfg(not(feature = "unaligned_access"))]
pub fn t1ha0_32be(data: &[u8], seed: u64) -> u64 {
    if !aligned_to::<u32, _>(data.as_ptr()) {
        unsafe { t1h0_body::<BigEndianUnaligned<u32>>(data, seed) }
    } else {
        unsafe { t1h0_body::<BigEndianAligned<u32>>(data, seed) }
    }
}

#[inline(always)]
unsafe fn t1h0_body<T>(data: &[u8], seed: u64) -> u64
where
    T: MemoryModel<Item = u32>,
{
    let mut len = data.len();
    let mut a = rot32(len as u32, 17).wrapping_add(seed as u32);
    let mut b = (len as u32) ^ ((seed >> 32) as u32);
    let mut v = data.as_ptr() as *const u32;

    if unlikely(len > 16) {
        let mut c = !a;
        let mut d = rot32(b, 5);
        let detent = (v as *const u8).offset(len as isize - 15) as usize;

        while likely((v as usize) < detent) {
            let w0 = T::fetch(v.offset(0));
            let w1 = T::fetch(v.offset(1));
            let w2 = T::fetch(v.offset(2));
            let w3 = T::fetch(v.offset(3));
            v = v.add(4);
            prefetch(v, 16);

            let d13 = w1.wrapping_add(rot32(w3.wrapping_add(d), 17));
            let c02 = w0 ^ rot32(w2.wrapping_add(c), 11);
            d ^= rot32(a.wrapping_add(w0), 3);
            c ^= rot32(b.wrapping_add(w1), 7);
            b = PRIME32_1.wrapping_mul(c02.wrapping_add(w3));
            a = PRIME32_0.wrapping_mul(d13 ^ w2);
        }

        c = c.wrapping_add(a);
        d = d.wrapping_add(b);
        a ^= rot32(c, 16).wrapping_add(d).wrapping_mul(PRIME32_6);
        b ^= c.wrapping_add(rot32(d, 16)).wrapping_mul(PRIME32_5);
        len &= 15;
    }

    match len {
        16 | 15 | 14 | 13 => {
            mixup32(&mut a, &mut b, T::fetch(v.offset(0)), PRIME32_4);
            mixup32(&mut b, &mut a, T::fetch(v.offset(1)), PRIME32_3);
            mixup32(&mut a, &mut b, T::fetch(v.offset(2)), PRIME32_2);
            mixup32(
                &mut b,
                &mut a,
                T::tail(v.offset(3), len as isize),
                PRIME32_1,
            );
            final32(a, b)
        }
        12 | 11 | 10 | 9 => {
            mixup32(&mut b, &mut a, T::fetch(v.offset(0)), PRIME32_3);
            mixup32(&mut a, &mut b, T::fetch(v.offset(1)), PRIME32_2);
            mixup32(
                &mut b,
                &mut a,
                T::tail(v.offset(2), len as isize),
                PRIME32_1,
            );
            final32(a, b)
        }
        8 | 7 | 6 | 5 => {
            mixup32(&mut a, &mut b, T::fetch(v.offset(0)), PRIME32_2);
            mixup32(
                &mut b,
                &mut a,
                T::tail(v.offset(1), len as isize),
                PRIME32_1,
            );
            final32(a, b)
        }
        4 | 3 | 2 | 1 => {
            mixup32(
                &mut b,
                &mut a,
                T::tail(v.offset(0), len as isize),
                PRIME32_1,
            );
            final32(a, b)
        }
        0 => final32(a, b),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selfcheck::selfcheck;

    const T1HA_REFVAL_32LE: [u64; 81] = [
        0,
        0xC92229C10FAEA50E,
        0x3DF1354B0DFDC443,
        0x968F016D60417BB3,
        0x85AAFB50C6DA770F,
        0x66CCE3BB6842C7D6,
        0xDDAA39C11537C226,
        0x35958D281F0C9C8C,
        0x8C5D64B091DE608E,
        0x4094DF680D39786B,
        0x1014F4AA2A2EDF4D,
        0x39D21891615AA310,
        0x7EF51F67C398C7C4,
        0x06163990DDBF319D,
        0xE229CAA00C8D6F3F,
        0xD2240B4B0D54E0F5,
        0xEA2E7E905DDEAF94,
        0x8D4F8A887183A5CE,
        0x44337F9A63C5820C,
        0x94938D1E86A9B797,
        0x96E9CABA5CA210CC,
        0x6EFBB9CC9E8F7708,
        0x3D12EA0282FB8BBC,
        0x5DA781EE205A2C48,
        0xFA4A51A12677FE12,
        0x81D5F04E20660B28,
        0x57258D043BCD3841,
        0x5C9BEB62059C1ED2,
        0x57A02162F9034B33,
        0xBA2A13E457CE19B8,
        0xE593263BF9451F3A,
        0x0BC1175539606BC5,
        0xA3E2929E9C5F289F,
        0x86BDBD06835E35F7,
        0xA180950AB48BAADC,
        0x7812C994D9924028,
        0x308366011415F46B,
        0x77FE9A9991C5F959,
        0x925C340B70B0B1E3,
        0xCD9C5BA4C41E2E10,
        0x7CC4E7758B94CD93,
        0x898B235962EA4625,
        0xD7E3E5BF22893286,
        0x396F4CDD33056C64,
        0x740AB2E32F17CD9F,
        0x60D12FF9CD15B321,
        0xBEE3A6C9903A81D8,
        0xB47040913B33C35E,
        0x19EE8C2ACC013CFF,
        0x5DEC94C5783B55C4,
        0x78DC122D562C5F1D,
        0x6520F008DA1C181E,
        0x77CAF155A36EBF7C,
        0x0A09E02BDB883CA6,
        0xFD5D9ADA7E3FB895,
        0xC6F5FDD9EEAB83B5,
        0x84589BB29F52A92A,
        0x9B2517F13F8E9814,
        0x6F752AF6A52E31EC,
        0x8E717799E324CE8A,
        0x84D90AEF39262D58,
        0x79C27B13FC28944D,
        0xE6D6DF6438E0044A,
        0x51B603E400D79CA4,
        0x6A902B28C588B390,
        0x8D7F8DE9E6CB1D83,
        0xCF1A4DC11CA7F044,
        0xEF02E43C366786F1,
        0x89915BCDBCFBE30F,
        0x5928B306F1A9CC7F,
        0xA8B59092996851C5,
        0x22050A20427E8B25,
        0x6E6D64018941E7EE,
        0x9798C898B81AE846,
        0x80EF218CDC30124A,
        0xFCE45E60D55B0284,
        0x4010E735D3147C35,
        0xEB647D999FD8DC7E,
        0xD3544DCAB14FE907,
        0xB588B27D8438700C,
        0xA49EBFC43E057A4C,
    ];

    const T1HA_REFVAL_32BE: [u64; 81] = [
        0,
        0xC92229C10FAEA50E,
        0x0FE212630DD87E0F,
        0x968F016D60417BB3,
        0xE6B12B2C889913AB,
        0xAA3787887A9DA368,
        0x06EE7202D53CEF39,
        0x6149AFB2C296664B,
        0x86C893210F9A5805,
        0x8379E5DA988AA04C,
        0x24763AA7CE411A60,
        0x9CF9C64B395A4CF8,
        0xFFC192C338DDE904,
        0x094575BAB319E5F5,
        0xBBBACFE7728C6511,
        0x36B8C3CEBE4EF409,
        0xAA0BA8A3397BA4D0,
        0xF9F85CF7124EE653,
        0x3ADF4F7DF2A887AE,
        0xAA2A0F5964AA9A7A,
        0xF18B563F42D36EB8,
        0x034366CEF8334F5C,
        0xAE2E85180E330E5F,
        0xA5CE9FBFDF5C65B8,
        0x5E509F25A9CA9B0B,
        0xE30D1358C2013BD2,
        0xBB3A04D5EB8111FE,
        0xB04234E82A15A28D,
        0x87426A56D0EA0E2F,
        0x095086668E07F9F8,
        0xF4CD3A43B6A6AEA5,
        0x73F9B9B674D472A6,
        0x558344229A1E4DCF,
        0x0AD4C95B2279181A,
        0x5E3D19D80821CA6B,
        0x652492D25BEBA258,
        0xEFA84B02EAB849B1,
        0x81AD2D253059AC2C,
        0x1400CCB0DFB2F457,
        0x5688DC72A839860E,
        0x67CC130E0FD1B0A7,
        0x0A851E3A94E21E69,
        0x2EA0000B6A073907,
        0xAE9776FF9BF1D02E,
        0xC0A96B66B160631C,
        0xA93341DE4ED7C8F0,
        0x6FBADD8F5B85E141,
        0xB7D295F1C21E0CBA,
        0x6D6114591B8E434F,
        0xF5B6939B63D97BE7,
        0x3C80D5053F0E5DB4,
        0xAC520ACC6B73F62D,
        0xD1051F5841CF3966,
        0x62245AEA644AE760,
        0x0CD56BE15497C62D,
        0x5BB93435C4988FB6,
        0x5FADB88EB18DB512,
        0xC897CAE2242475CC,
        0xF1A094EF846DC9BB,
        0x2B1D8B24924F79B6,
        0xC6DF0C0E8456EB53,
        0xE6A40128303A9B9C,
        0x64D37AF5EFFA7BD9,
        0x90FEB70A5AE2A598,
        0xEC3BA5F126D9FF4B,
        0x3121C8EC3AC51B29,
        0x3B41C4D422166EC1,
        0xB4878DDCBF48ED76,
        0x5CB850D77CB762E4,
        0x9A27A43CC1DD171F,
        0x2FDFFC6F99CB424A,
        0xF54A57E09FDEA7BB,
        0x5F78E5EE2CAB7039,
        0xB8BA95883DB31CBA,
        0x131C61EB84AF86C3,
        0x84B1F64E9C613DA7,
        0xE94C1888C0C37C02,
        0xEA08F8BFB2039CDE,
        0xCCC6D04D243EC753,
        0x8977D105298B0629,
        0x7AAA976494A5905E,
    ];

    #[test]
    fn test_t1ha0_32le() {
        selfcheck(t1ha0_32le, &T1HA_REFVAL_32LE[..])
    }

    #[test]
    fn test_t1ha0_32be() {
        selfcheck(t1ha0_32be, &T1HA_REFVAL_32BE[..])
    }
}
