#![allow(clippy::cast_ptr_alignment)]

use crate::{bits::*, nightly::*};

/// An implementation of `t1ha2` stream hasher.
#[derive(Debug, Default)]
pub struct T1ha2Hasher {
    state: State,
    buffer: [u8; 32],
    partial: usize,
    total: usize,
}

#[derive(Debug, Default)]
struct State {
    a: u64,
    b: u64,
    c: u64,
    d: u64,
}

impl State {
    #[inline(always)]
    pub fn init_ab(&mut self, x: u64, y: u64) {
        self.a = x;
        self.b = y;
    }

    #[inline(always)]
    pub fn init_cd(&mut self, x: u64, y: u64) {
        self.c = rot64(y, 23).wrapping_add(!x);
        self.d = (!y).wrapping_add(rot64(x, 19));
    }

    #[inline(always)]
    fn squash(&mut self) {
        self.a ^= PRIME_6.wrapping_mul(self.c.wrapping_add(rot64(self.d, 23)));
        self.b ^= PRIME_5.wrapping_mul(rot64(self.c, 19).wrapping_add(self.d));
    }
}

/// The at-once variant with 64-bit result
#[cfg(feature = "unaligned_access")]
pub fn t1ha2_atonce(data: &[u8], seed: u64) -> u64 {
    t1ha2_atonce_body::<LittenEndianUnaligned<u64>>(data, seed)
}

/// The at-once variant with 64-bit result
#[cfg(not(feature = "unaligned_access"))]
pub fn t1ha2_atonce(data: &[u8], seed: u64) -> u64 {
    if !aligned_to::<u64, _>(data.as_ptr()) {
        t1ha2_atonce_body::<LittenEndianUnaligned<u64>>(data, seed)
    } else {
        t1ha2_atonce_body::<LittenEndianAligned<u64>>(data, seed)
    }
}

fn t1ha2_atonce_body<T: MemoryModel<Item = u64>>(mut data: &[u8], seed: u64) -> u64 {
    let mut state = State::default();
    let len = data.len();

    state.init_ab(seed, len as u64);

    if unlikely(len > 32) {
        state.init_cd(seed, len as u64);
        unsafe {
            data = t1ha2_loop::<T>(&mut state, data);
        }
        state.squash();
    }
    unsafe { t1ha2_tail_ab::<T>(&mut state, data) }
}

/// The at-once variant with 128-bit result.
#[cfg(feature = "unaligned_access")]
pub fn t1ha2_atonce128(data: &[u8], seed: u64) -> u128 {
    t1ha2_atonce128_body::<LittenEndianUnaligned<u64>>(data, seed)
}

/// The at-once variant with 128-bit result.
#[cfg(not(feature = "unaligned_access"))]
pub fn t1ha2_atonce128(data: &[u8], seed: u64) -> u128 {
    if !aligned_to::<u64, _>(data.as_ptr()) {
        t1ha2_atonce128_body::<LittenEndianUnaligned<u64>>(data, seed)
    } else {
        t1ha2_atonce128_body::<LittenEndianAligned<u64>>(data, seed)
    }
}

fn t1ha2_atonce128_body<T: MemoryModel<Item = u64>>(mut data: &[u8], seed: u64) -> u128 {
    let mut state = State::default();
    let len = data.len();

    state.init_ab(seed, len as u64);
    state.init_cd(seed, len as u64);

    if unlikely(len > 32) {
        unsafe {
            data = t1ha2_loop::<T>(&mut state, data);
        }
    }
    unsafe { t1ha2_tail_abcd::<T>(&mut state, data) }
}

impl T1ha2Hasher {
    pub fn with_seeds(seed_x: u64, seed_y: u64) -> Self {
        let mut h = Self::default();

        h.state.init_ab(seed_x, seed_y);
        h.state.init_cd(seed_x, seed_y);
        h
    }

    pub fn update(&mut self, mut data: &[u8]) {
        let mut len = data.len();

        self.total += data.len();

        if self.partial > 0 {
            let left = 32 - self.partial;
            let chunk = len.min(left);
            self.append(&data[..chunk]);
            if self.partial < 32 {
                debug_assert!(left >= len);

                return;
            }

            self.partial = 0;

            data = &data[chunk..];
            len -= chunk;

            unsafe {
                t1ha2_update::<LittenEndianAligned<u64>>(
                    &mut self.state,
                    self.buffer.as_ptr() as *const u64,
                )
            }
        }

        if len >= 32 {
            unsafe {
                data = if cfg!(feature = "unaligned_access") || !aligned_to::<u64, _>(data.as_ptr())
                {
                    t1ha2_loop::<LittenEndianUnaligned<u64>>(&mut self.state, data)
                } else {
                    t1ha2_loop::<LittenEndianAligned<u64>>(&mut self.state, data)
                };
            }

            len &= 31;
        }

        if len > 0 {
            self.append(data)
        }
    }

    fn append(&mut self, data: &[u8]) {
        debug_assert!(self.partial + data.len() <= self.buffer.len());

        let buf = &mut self.buffer[self.partial..self.partial + data.len()];

        buf.copy_from_slice(data);

        self.partial += data.len();
    }

    pub fn finish128(&mut self) -> u128 {
        let mut bits = ((self.total as u64) << 3) ^ (1u64 << 63);

        if cfg!(target_endian = "big") {
            bits = bits.to_le();
        }

        self.update(&bits.to_ne_bytes());

        unsafe {
            t1ha2_tail_abcd::<LittenEndianAligned<u64>>(
                &mut self.state,
                &self.buffer[..self.partial],
            )
        }
    }

    pub fn finish(&mut self) -> u64 {
        let mut bits = ((self.total as u64) << 3) ^ (1u64 << 63);

        if cfg!(target_endian = "big") {
            bits = bits.to_le();
        }

        self.update(&bits.to_ne_bytes());

        self.state.squash();

        unsafe {
            t1ha2_tail_ab::<LittenEndianAligned<u64>>(&mut self.state, &self.buffer[..self.partial])
        }
    }
}

#[inline(always)]
unsafe fn t1ha2_update<T: MemoryModel<Item = u64>>(state: &mut State, v: *const u64) {
    let w0 = T::fetch(v.offset(0));
    let w1 = T::fetch(v.offset(1));
    let w2 = T::fetch(v.offset(2));
    let w3 = T::fetch(v.offset(3));

    let d02 = w0.wrapping_add(rot64(w2.wrapping_add(state.d), 56));
    let c13 = w1.wrapping_add(rot64(w3.wrapping_add(state.c), 19));

    state.d ^= state.b.wrapping_add(rot64(w1, 38));
    state.c ^= state.a.wrapping_add(rot64(w0, 57));
    state.b ^= PRIME_6.wrapping_mul(c13.wrapping_add(w2));
    state.a ^= PRIME_5.wrapping_mul(d02.wrapping_add(w3));
}

#[inline(always)]
unsafe fn t1ha2_loop<'a, T: MemoryModel<Item = u64>>(
    state: &mut State,
    data: &'a [u8],
) -> &'a [u8] {
    let p = data.as_ptr();
    let len = data.len();
    let detent = p as usize + len - 31;
    let mut v = p as *const u64;

    while likely((v as usize) < detent) {
        let d = v;
        v = v.add(4);
        prefetch(p);

        t1ha2_update::<T>(state, d)
    }

    &data[(v as usize) - (p as usize)..]
}

#[inline(always)]
unsafe fn t1ha2_tail_ab<T: MemoryModel<Item = u64>>(state: &mut State, data: &[u8]) -> u64 {
    let v = data.as_ptr() as *const u64;
    let len = data.len();

    match len {
        25..=32 => {
            mixup64(&mut state.a, &mut state.b, T::fetch(v.offset(0)), PRIME_4);
            mixup64(&mut state.b, &mut state.a, T::fetch(v.offset(1)), PRIME_3);
            mixup64(&mut state.a, &mut state.b, T::fetch(v.offset(2)), PRIME_2);
            mixup64(
                &mut state.b,
                &mut state.a,
                T::tail(v.offset(3), len as isize),
                PRIME_1,
            );
            final64(state.a, state.b)
        }
        17..=24 => {
            mixup64(&mut state.b, &mut state.a, T::fetch(v.offset(0)), PRIME_3);
            mixup64(&mut state.a, &mut state.b, T::fetch(v.offset(1)), PRIME_2);
            mixup64(
                &mut state.b,
                &mut state.a,
                T::tail(v.offset(2), len as isize),
                PRIME_1,
            );
            final64(state.a, state.b)
        }
        9..=16 => {
            mixup64(&mut state.a, &mut state.b, T::fetch(v.offset(0)), PRIME_2);
            mixup64(
                &mut state.b,
                &mut state.a,
                T::tail(v.offset(1), len as isize),
                PRIME_1,
            );
            final64(state.a, state.b)
        }
        1..=8 => {
            mixup64(
                &mut state.b,
                &mut state.a,
                T::tail(v, len as isize),
                PRIME_1,
            );
            final64(state.a, state.b)
        }
        0 => final64(state.a, state.b),
        _ => unreachable!(),
    }
}

#[inline(always)]
unsafe fn t1ha2_tail_abcd<T: MemoryModel<Item = u64>>(state: &mut State, data: &[u8]) -> u128 {
    let v = data.as_ptr() as *const u64;
    let len = data.len();

    match len {
        25..=32 => {
            mixup64(&mut state.a, &mut state.d, T::fetch(v.offset(0)), PRIME_4);
            mixup64(&mut state.b, &mut state.a, T::fetch(v.offset(1)), PRIME_3);
            mixup64(&mut state.c, &mut state.b, T::fetch(v.offset(2)), PRIME_2);
            mixup64(
                &mut state.d,
                &mut state.c,
                T::tail(v.offset(3), len as isize),
                PRIME_1,
            );
            final128(state)
        }
        17..=24 => {
            mixup64(&mut state.b, &mut state.a, T::fetch(v.offset(0)), PRIME_3);
            mixup64(&mut state.c, &mut state.b, T::fetch(v.offset(1)), PRIME_2);
            mixup64(
                &mut state.d,
                &mut state.c,
                T::tail(v.offset(2), len as isize),
                PRIME_1,
            );
            final128(state)
        }
        9..=16 => {
            mixup64(&mut state.c, &mut state.b, T::fetch(v.offset(0)), PRIME_2);
            mixup64(
                &mut state.d,
                &mut state.c,
                T::tail(v.offset(1), len as isize),
                PRIME_1,
            );
            final128(state)
        }
        1..=8 => {
            mixup64(
                &mut state.d,
                &mut state.c,
                T::tail(v, len as isize),
                PRIME_1,
            );
            final128(state)
        }
        0 => final128(state),
        _ => unreachable!(),
    }
}

#[inline(always)]
fn final128(state: &mut State) -> u128 {
    mixup64(
        &mut state.a,
        &mut state.b,
        rot64(state.c, 41) ^ state.d,
        PRIME_0,
    );
    mixup64(
        &mut state.b,
        &mut state.c,
        rot64(state.d, 23) ^ state.a,
        PRIME_6,
    );
    mixup64(
        &mut state.c,
        &mut state.d,
        rot64(state.a, 19) ^ state.b,
        PRIME_5,
    );
    mixup64(
        &mut state.d,
        &mut state.a,
        rot64(state.b, 31) ^ state.c,
        PRIME_4,
    );

    u128::from(state.c.wrapping_add(state.d))
        .wrapping_shl(64)
        .wrapping_add(u128::from(state.a ^ state.b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selfcheck::selfcheck;

    const T1HA_REFVAL_2ATONCE: [u64; 81] = [
        0,
        0x772C7311BE32FF42,
        0x444753D23F207E03,
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
        0x25A7201C85D9E2A3,
        0x911573EDA15299AA,
        0x5C0062B669E18E4C,
        0x17734ADE08D54E28,
        0xFFF036E33883F43B,
        0xFE0756E7777DF11E,
        0x37972472D023F129,
        0x6CFCE201B55C7F57,
        0xE019D1D89F02B3E1,
        0xAE5CC580FA1BB7E6,
        0x295695FB7E59FC3A,
        0x76B6C820A40DD35E,
        0xB1680A1768462B17,
        0x2FB6AF279137DADA,
        0x28FB6B4366C78535,
        0xEC278E53924541B1,
        0x164F8AAB8A2A28B5,
        0xB6C330AEAC4578AD,
        0x7F6F371070085084,
        0x94DEAD60C0F448D3,
        0x99737AC232C559EF,
        0x6F54A6F9CA8EDD57,
        0x979B01E926BFCE0C,
        0xF7D20BC85439C5B4,
        0x64EDB27CD8087C12,
        0x11488DE5F79C0BE2,
        0x25541DDD1680B5A4,
        0x8B633D33BE9D1973,
        0x404A3113ACF7F6C6,
        0xC59DBDEF8550CD56,
        0x039D23C68F4F992C,
        0x5BBB48E4BDD6FD86,
        0x41E312248780DF5A,
        0xD34791CE75D4E94F,
        0xED523E5D04DCDCFF,
        0x7A6BCE0B6182D879,
        0x21FB37483CAC28D8,
        0x19A1B66E8DA878AD,
        0x6F804C5295B09ABE,
        0x2A4BE5014115BA81,
        0xA678ECC5FC924BE0,
        0x50F7A54A99A36F59,
        0x0FD7E63A39A66452,
        0x5AB1B213DD29C4E4,
        0xF3ED80D9DF6534C5,
        0xC736B12EF90615FD,
    ];

    const T1HA_REFVAL_2ATONCE128: [u64; 81] = [
        0x4EC7F6A48E33B00A,
        0xB7B7FAA5BD7D8C1E,
        0x3269533F66534A76,
        0x6C3EC6B687923BFC,
        0xC096F5E7EFA471A9,
        0x79D8AFB550CEA471,
        0xCEE0507A20FD5119,
        0xFB04CFFC14A9F4BF,
        0xBD4406E923807AF2,
        0x375C02FF11010491,
        0xA6EA4C2A59E173FF,
        0xE0A606F0002CADDF,
        0xE13BEAE6EBC07897,
        0xF069C2463E48EA10,
        0x75BEE1A97089B5FA,
        0x378F22F8DE0B8085,
        0x9C726FC4D53D0D8B,
        0x71F6130A2D08F788,
        0x7A9B20433FF6CF69,
        0xFF49B7CD59BF6D61,
        0xCCAAEE0D1CA9C6B3,
        0xC77889D86039D2AD,
        0x7B378B5BEA9B0475,
        0x6520BFA79D59AD66,
        0x2441490CB8A37267,
        0xA715A66B7D5CF473,
        0x9AE892C88334FD67,
        0xD2FFE9AEC1D2169A,
        0x790B993F18B18CBB,
        0xA0D02FBCF6A7B1AD,
        0xA90833E6F151D0C1,
        0x1AC7AFA37BD79BE0,
        0xD5383628B2881A24,
        0xE5526F9D63F9F8F1,
        0xC1F165A01A6D1F4D,
        0x6CCEF8FF3FCFA3F2,
        0x2030F18325E6DF48,
        0x289207230E3FB17A,
        0x077B66F713A3C4B9,
        0x9F39843CAF871754,
        0x512FDA0F808ACCF3,
        0xF4D9801CD0CD1F14,
        0x28A0C749ED323638,
        0x94844CAFA671F01C,
        0xD0E261876B8ACA51,
        0x8FC2A648A4792EA2,
        0x8EF87282136AF5FE,
        0x5FE6A54A9FBA6B40,
        0xA3CC5B8FE6223D54,
        0xA8C3C0DD651BB01C,
        0x625E9FDD534716F3,
        0x1AB2604083C33AC5,
        0xDE098853F8692F12,
        0x4B0813891BD87624,
        0x4AB89C4553D182AD,
        0x92C15AA2A3C27ADA,
        0xFF2918D68191F5D9,
        0x06363174F641C325,
        0x667112ADA74A2059,
        0x4BD605D6B5E53D7D,
        0xF2512C53663A14C8,
        0x21857BCB1852667C,
        0xAFBEBD0369AEE228,
        0x7049340E48FBFD6B,
        0x50710E1924F46954,
        0x869A75E04A976A3F,
        0x5A41ABBDD6373889,
        0xA781778389B4B188,
        0x21A3AFCED6C925B6,
        0x107226192EC10B42,
        0x62A862E84EC2F9B1,
        0x2B15E91659606DD7,
        0x613934D1F9EC5A42,
        0x4DC3A96DC5361BAF,
        0xC80BBA4CB5F12903,
        0x3E3EDAE99A7D6987,
        0x8F97B2D55941DCB0,
        0x4C9787364C3E4EC1,
        0xEF0A2D07BEA90CA7,
        0x5FABF32C70AEEAFB,
        0x3356A5CFA8F23BF4,
    ];

    const T1HA_REFVAL_2STREAM: [u64; 81] = [
        0x3C8426E33CB41606,
        0xFD74BE70EE73E617,
        0xF43DE3CDD8A20486,
        0x882FBCB37E8EA3BB,
        0x1AA2CDD34CAA3D4B,
        0xEE755B2BFAE07ED5,
        0xD4E225250D92E213,
        0xA09B49083205965B,
        0xD47B21724EF9EC9E,
        0xAC888FC3858CEE11,
        0x94F820D85736F244,
        0x1707951CCA920932,
        0x8E0E45603F7877F0,
        0x9FD2592C0E3A7212,
        0x9A66370F3AE3D427,
        0xD33382D2161DE2B7,
        0x9A35BE079DA7115F,
        0x73457C7FF58B4EC3,
        0xBE8610BD53D7CE98,
        0x65506DFE5CCD5371,
        0x286A321AF9D5D9FA,
        0xB81EF9A7EF3C536D,
        0x2CFDB5E6825C6E86,
        0xB2A58CBFDFDD303A,
        0xD26094A42B950635,
        0xA34D666A5F02AD9A,
        0x0151E013EBCC72E5,
        0x9254A6EA7FCB6BB5,
        0x10C9361B3869DC2B,
        0xD7EC55A060606276,
        0xA2FF7F8BF8976FFD,
        0xB5181BB6852DCC88,
        0x0EE394BB6178BAFF,
        0x3A8B4B400D21B89C,
        0xEC270461970960FD,
        0x615967FAB053877E,
        0xFA51BF1CFEB4714C,
        0x29FDA8383070F375,
        0xC3B663061BC52EDA,
        0x192BBAF1F1A57923,
        0x6D193B52F93C53AF,
        0x7F6F5639FE87CA1E,
        0x69F7F9140B32EDC8,
        0xD0F2416FB24325B6,
        0x62C0E37FEDD49FF3,
        0x57866A4B809D373D,
        0x9848D24BD935E137,
        0xDFC905B66734D50A,
        0x9A938DD194A68529,
        0x8276C44DF0625228,
        0xA4B35D00AD67C0AB,
        0x3D9CB359842DB452,
        0x4241BFA8C23B267F,
        0x650FA517BEF15952,
        0x782DE2ABD8C7B1E1,
        0x4EAE456166CA3E15,
        0x40CDF3A02614E337,
        0xAD84092C46102172,
        0x0C68479B03F9A167,
        0x7E1BA046749E181C,
        0x3F3AB41A697382C1,
        0xC5E5DD6586EBFDC4,
        0xFF926CD4EB02555C,
        0x035CFE67F89E709B,
        0x89F06AB6464A1B9D,
        0x8EFF58F3F7DEA758,
        0x8B54AC657902089F,
        0xC6C4F1F9F8DA4D64,
        0xBDB729048AAAC93A,
        0xEA76BA628F5E5CD6,
        0x742159B728B8A979,
        0x6D151CD3C720E53D,
        0xE97FFF9368FCDC42,
        0xCA5B38314914FBDA,
        0xDD92C91D8B858EAE,
        0x66E5F07CF647CBF2,
        0xD4CF9B42F4985AFB,
        0x72AE17AC7D92F6B7,
        0xB8206B22AB0472E1,
        0x385876B5CFD42479,
        0x03294A249EBE6B26,
    ];

    const T1HA_REFVAL_2STREAM128: [u64; 81] = [
        0xCD2801D3B92237D6,
        0x10E4D47BD821546D,
        0x9100704B9D65CD06,
        0xD6951CB4016313EF,
        0x24DB636F96F474DA,
        0x3F4AF7DF3C49E422,
        0xBFF25B8AF143459B,
        0xA157EC13538BE549,
        0xD3F5F52C47DBD419,
        0x0EF3D7D735AF1575,
        0x46B7B892823F7B1B,
        0xEE22EA4655213289,
        0x56AD76F02FE929BC,
        0x9CF6CD1AC886546E,
        0xAF45CE47AEA0B933,
        0x535F9DC09F3996B7,
        0x1F0C3C01694AE128,
        0x18495069BE0766F7,
        0x37E5FFB3D72A4CB1,
        0x6D6C2E9299F30709,
        0x4F39E693F50B41E3,
        0xB11FC4EF0658E116,
        0x48BFAACB78E5079B,
        0xE1B4C89C781B3AD0,
        0x81D2F34888D333A1,
        0xF6D02270D2EA449C,
        0xC884C3C2C3CE1503,
        0x711AE16BA157A9B9,
        0x1E6140C642558C9D,
        0x35AB3D238F5DC55B,
        0x33F07B6AEF051177,
        0xE57336776EEFA71C,
        0x6D445F8318BA3752,
        0xD4F5F6631934C988,
        0xD5E260085727C4A2,
        0x5B54B41EC180B4FA,
        0x7F5D75769C15A898,
        0xAE5A6DB850CA33C6,
        0x038CCB8044663403,
        0xDA16310133DC92B8,
        0x6A2FFB7AB2B7CE2B,
        0xDC1832D9229BAE20,
        0x8C62C479F5ABC9E4,
        0x5EB7B617857C9CCB,
        0xB79CF7D749A1E80D,
        0xDE7FAC3798324FD3,
        0x8178911813685D06,
        0x6A726CBD394D4410,
        0x6CBE6B3280DA1113,
        0x6829BA4410CF1148,
        0xFA7E417EB26C5BC6,
        0x22ED87884D6E3A49,
        0x15F1472D5115669D,
        0x2EA0B4C8BF69D318,
        0xDFE87070AA545503,
        0x6B4C14B5F7144AB9,
        0xC1ED49C06126551A,
        0x351919FC425C3899,
        0x7B569C0FA6F1BD3E,
        0x713AC2350844CFFD,
        0xE9367F9A638C2FF3,
        0x97F17D325AEA0786,
        0xBCB907CC6CF75F91,
        0x0CB7517DAF247719,
        0xBE16093CC45BE8A9,
        0x786EEE97359AD6AB,
        0xB7AFA4F326B97E78,
        0x2694B67FE23E502E,
        0x4CB492826E98E0B4,
        0x838D119F74A416C7,
        0x70D6A91E4E5677FD,
        0xF3E4027AD30000E6,
        0x9BDF692795807F77,
        0x6A371F966E034A54,
        0x8789CF41AE4D67EF,
        0x02688755484D60AE,
        0xD5834B3A4BF5CE42,
        0x9405FC61440DE25D,
        0x35EB280A157979B6,
        0x48D40D6A525297AC,
        0x6A87DC185054BADA,
    ];

    #[test]
    fn test_t1ha2_atonce() {
        selfcheck(t1ha2_atonce, &T1HA_REFVAL_2ATONCE[..])
    }

    #[test]
    fn test_t1ha2_atonce128() {
        selfcheck(
            |data, seed| t1ha2_atonce128(data, seed) as u64,
            &T1HA_REFVAL_2ATONCE128[..],
        )
    }

    #[test]
    fn test_t1ha2_stream() {
        selfcheck(
            |data, seed| {
                let mut h = T1ha2Hasher::with_seeds(seed, seed);
                h.update(data);
                h.finish()
            },
            &T1HA_REFVAL_2STREAM[..],
        )
    }

    #[test]
    fn test_t1ha2_stream128() {
        selfcheck(
            |data, seed| {
                let mut h = T1ha2Hasher::with_seeds(seed, seed);
                h.update(data);
                h.finish128() as u64
            },
            &T1HA_REFVAL_2STREAM128[..],
        )
    }
}
