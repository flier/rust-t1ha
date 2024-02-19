//!
//! t1ha1 = 64-bit, BASELINE FAST PORTABLE HASH:
//!
//!   - Runs faster on 64-bit platforms in other cases may runs slowly.
//!   - Portable and stable, returns same 64-bit result
//!     on all architectures and CPUs.
//!   - Unfortunately it fails the "strict avalanche criteria",
//!     see test results at https://github.com/demerphq/smhasher.
//!
//!     This flaw is insignificant for the t1ha1() purposes and imperceptible
//!     from a practical point of view.
//!     However, nowadays this issue has resolved in the next t1ha2(),
//!     that was initially planned to providing a bit more quality.
#![allow(clippy::cast_ptr_alignment, clippy::many_single_char_names)]

use crate::{bits::*, nightly::*};

/// The little-endian variant for 64-bit CPU.
#[cfg(feature = "unaligned_access")]
pub fn t1ha1_le(data: &[u8], seed: u64) -> u64 {
    unsafe { t1h1_body::<LittenEndianUnaligned<u64>>(data, seed) }
}

/// The little-endian variant for 64-bit CPU.
#[cfg(not(feature = "unaligned_access"))]
pub fn t1ha1_le(data: &[u8], seed: u64) -> u64 {
    if !aligned_to::<u64, _>(data.as_ptr()) {
        unsafe { t1h1_body::<LittenEndianUnaligned<u64>>(data, seed) }
    } else {
        unsafe { t1h1_body::<LittenEndianAligned<u64>>(data, seed) }
    }
}

/// The big-endian variant for 64-bit CPU.
#[cfg(feature = "unaligned_access")]
pub fn t1ha1_be(data: &[u8], seed: u64) -> u64 {
    unsafe { t1h0_body::<BigEndianUnaligned<u64>>(data, seed) }
}

/// The big-endian variant for 64-bit CPU.
#[cfg(not(feature = "unaligned_access"))]
pub fn t1ha1_be(data: &[u8], seed: u64) -> u64 {
    if !aligned_to::<u64, _>(data.as_ptr()) {
        unsafe { t1h1_body::<BigEndianUnaligned<u64>>(data, seed) }
    } else {
        unsafe { t1h1_body::<BigEndianAligned<u64>>(data, seed) }
    }
}

#[inline(always)]
unsafe fn t1h1_body<T>(data: &[u8], seed: u64) -> u64
where
    T: MemoryModel<Item = u64>,
{
    let mut len = data.len() as u64;
    let mut a = seed;
    let mut b = len;
    let mut v = data.as_ptr() as *const u64;

    if unlikely(len > 32) {
        let mut c = rot64(len, 17).wrapping_add(seed);
        let mut d = len ^ rot64(seed, 17);
        let detent = v as usize + len as usize - 31;

        while likely((v as usize) < detent) {
            let w0 = T::fetch(v.offset(0));
            let w1 = T::fetch(v.offset(1));
            let w2 = T::fetch(v.offset(2));
            let w3 = T::fetch(v.offset(3));
            v = v.add(4);
            prefetch(v);

            let d02 = w0 ^ rot64(w2.wrapping_add(d), 17);
            let c13 = w1 ^ rot64(w3.wrapping_add(c), 17);
            d = d.wrapping_sub(b ^ rot64(w1, 31));
            c = c.wrapping_add(a ^ rot64(w0, 41));
            b ^= PRIME_0.wrapping_mul(c13.wrapping_add(w2));
            a ^= PRIME_1.wrapping_mul(d02.wrapping_add(w3));
        }

        a ^= PRIME_6.wrapping_mul(rot64(c, 17).wrapping_add(d));
        b ^= PRIME_5.wrapping_mul(c.wrapping_add(rot64(d, 17)));

        len &= 31;
    }

    match len {
        25..=32 => {
            b = b.wrapping_add(mux64(T::fetch(v.offset(0)), PRIME_4));
            a = a.wrapping_add(mux64(T::fetch(v.offset(1)), PRIME_3));
            b = b.wrapping_add(mux64(T::fetch(v.offset(2)), PRIME_2));
            a = a.wrapping_add(mux64(T::tail(v.offset(3), len as isize), PRIME_1));
            final_weak_avalanche(a, b)
        }
        17..=24 => {
            a = a.wrapping_add(mux64(T::fetch(v.offset(0)), PRIME_3));
            b = b.wrapping_add(mux64(T::fetch(v.offset(1)), PRIME_2));
            a = a.wrapping_add(mux64(T::tail(v.offset(2), len as isize), PRIME_1));
            final_weak_avalanche(a, b)
        }
        9..=16 => {
            b = b.wrapping_add(mux64(T::fetch(v.offset(0)), PRIME_2));
            a = a.wrapping_add(mux64(T::tail(v.offset(1), len as isize), PRIME_1));
            final_weak_avalanche(a, b)
        }
        1..=8 => {
            a = a.wrapping_add(mux64(T::tail(v, len as isize), PRIME_1));
            final_weak_avalanche(a, b)
        }
        0 => final_weak_avalanche(a, b),
        _ => unreachable!(),
    }
}

#[inline(always)]
fn final_weak_avalanche(a: u64, b: u64) -> u64 {
    mux64(rot64(a.wrapping_add(b), 17), PRIME_4).wrapping_add(mix64(a ^ b, PRIME_0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selfcheck::selfcheck;

    const T1HA_REFVAL_64LE: [u64; 81] = [
        0,
        0x6A580668D6048674,
        0xA2FE904AFF0D0879,
        0xE3AB9C06FAF4D023,
        0x6AF1C60874C95442,
        0xB3557E561A6C5D82,
        0x0AE73C696F3D37C0,
        0x5EF25F7062324941,
        0x9B784F3B4CE6AF33,
        0x6993BB206A74F070,
        0xF1E95DF109076C4C,
        0x4E1EB70C58E48540,
        0x5FDD7649D8EC44E4,
        0x559122C706343421,
        0x380133D58665E93D,
        0x9CE74296C8C55AE4,
        0x3556F9A5757AB6D0,
        0xF62751F7F25C469E,
        0x851EEC67F6516D94,
        0xED463EE3848A8695,
        0xDC8791FEFF8ED3AC,
        0x2569C744E1A282CF,
        0xF90EB7C1D70A80B9,
        0x68DFA6A1B8050A4C,
        0x94CCA5E8210D2134,
        0xF5CC0BEABC259F52,
        0x40DBC1F51618FDA7,
        0x0807945BF0FB52C6,
        0xE5EF7E09DE70848D,
        0x63E1DF35FEBE994A,
        0x2025E73769720D5A,
        0xAD6120B2B8A152E1,
        0x2A71D9F13959F2B7,
        0x8A20849A27C32548,
        0x0BCBC9FE3B57884E,
        0x0E028D255667AEAD,
        0xBE66DAD3043AB694,
        0xB00E4C1238F9E2D4,
        0x5C54BDE5AE280E82,
        0x0E22B86754BC3BC4,
        0x016707EBF858B84D,
        0x990015FBC9E095EE,
        0x8B9AF0A3E71F042F,
        0x6AA56E88BD380564,
        0xAACE57113E681A0F,
        0x19F81514AFA9A22D,
        0x80DABA3D62BEAC79,
        0x715210412CABBF46,
        0xD8FA0B9E9D6AA93F,
        0x6C2FC5A4109FD3A2,
        0x5B3E60EEB51DDCD8,
        0x0A7C717017756FE7,
        0xA73773805CA31934,
        0x4DBD6BB7A31E85FD,
        0x24F619D3D5BC2DB4,
        0x3E4AF35A1678D636,
        0x84A1A8DF8D609239,
        0x359C862CD3BE4FCD,
        0xCF3A39F5C27DC125,
        0xC0FF62F8FD5F4C77,
        0x5E9F2493DDAA166C,
        0x17424152BE1CA266,
        0xA78AFA5AB4BBE0CD,
        0x7BFB2E2CEF118346,
        0x647C3E0FF3E3D241,
        0x0352E4055C13242E,
        0x6F42FC70EB660E38,
        0x0BEBAD4FABF523BA,
        0x9269F4214414D61D,
        0x1CA8760277E6006C,
        0x7BAD25A859D87B5D,
        0xAD645ADCF7414F1D,
        0xB07F517E88D7AFB3,
        0xB321C06FB5FFAB5C,
        0xD50F162A1EFDD844,
        0x1DFD3D1924FBE319,
        0xDFAEAB2F09EF7E78,
        0xA7603B5AF07A0B1E,
        0x41CD044C0E5A4EE3,
        0xF64D2F86E813BF33,
        0xFF9FDB99305EB06A,
    ];

    const T1HA_REFVAL_64BE: [u64; 81] = [
        0,
        0x6A580668D6048674,
        0xDECC975A0E3B8177,
        0xE3AB9C06FAF4D023,
        0xE401FA8F1B6AF969,
        0x67DB1DAE56FB94E3,
        0x1106266A09B7A073,
        0x550339B1EF2C7BBB,
        0x290A2BAF590045BB,
        0xA182C1258C09F54A,
        0x137D53C34BE7143A,
        0xF6D2B69C6F42BEDC,
        0x39643EAF2CA2E4B4,
        0x22A81F139A2C9559,
        0x5B3D6AEF0AF33807,
        0x56E3F80A68643C08,
        0x9E423BE502378780,
        0xCDB0986F9A5B2FD5,
        0xD5B3C84E7933293F,
        0xE5FB8C90399E9742,
        0x5D393C1F77B2CF3D,
        0xC8C82F5B2FF09266,
        0xACA0230CA6F7B593,
        0xCB5805E2960D1655,
        0x7E2AD5B704D77C95,
        0xC5E903CDB8B9EB5D,
        0x4CC7D0D21CC03511,
        0x8385DF382CFB3E93,
        0xF17699D0564D348A,
        0xF77EE7F8274A4C8D,
        0xB9D8CEE48903BABE,
        0xFE0EBD2A82B9CFE9,
        0xB49FB6397270F565,
        0x173735C8C342108E,
        0xA37C7FBBEEC0A2EA,
        0xC13F66F462BB0B6E,
        0x0C04F3C2B551467E,
        0x76A9CB156810C96E,
        0x2038850919B0B151,
        0xCEA19F2B6EED647B,
        0x6746656D2FA109A4,
        0xF05137F221007F37,
        0x892FA9E13A3B4948,
        0x4D57B70D37548A32,
        0x1A7CFB3D566580E6,
        0x7CB30272A45E3FAC,
        0x137CCFFD9D51423F,
        0xB87D96F3B82DF266,
        0x33349AEE7472ED37,
        0x5CC0D3C99555BC07,
        0x4A8F4FA196D964EF,
        0xE82A0D64F281FBFA,
        0x38A1BAC2C36823E1,
        0x77D197C239FD737E,
        0xFB07746B4E07DF26,
        0xC8A2198E967672BD,
        0x5F1A146D143FA05A,
        0x26B877A1201AB7AC,
        0x74E5B145214723F8,
        0xE9CE10E3C70254BC,
        0x299393A0C05B79E8,
        0xFD2D2B9822A5E7E2,
        0x85424FEA50C8E50A,
        0xE6839E714B1FFFE5,
        0x27971CCB46F9112A,
        0xC98695A2E0715AA9,
        0x338E1CBB4F858226,
        0xFC6B5C5CF7A8D806,
        0x8973CAADDE8DA50C,
        0x9C6D47AE32EBAE72,
        0x1EBF1F9F21D26D78,
        0x80A9704B8E153859,
        0x6AFD20A939F141FB,
        0xC35F6C2B3B553EEF,
        0x59529E8B0DC94C1A,
        0x1569DF036EBC4FA1,
        0xDA32B88593C118F9,
        0xF01E4155FF5A5660,
        0x765A2522DCE2B185,
        0xCEE95554128073EF,
        0x60F072A5CA51DE2F,
    ];

    #[test]
    fn test_t1ha1_le() {
        selfcheck(t1ha1_le, &T1HA_REFVAL_64LE[..])
    }

    #[test]
    fn test_t1ha1_be() {
        selfcheck(t1ha1_be, &T1HA_REFVAL_64BE[..])
    }
}
