use std::vec::Vec;

const T1HA_TEST_PATTERN: [u8; 64] = [
    0, 1, 2, 3, 4, 5, 6, 7, 0xFF, 0x7F, 0x3F, 0x1F, 0xF, 8, 16, 32, 64, 0x80, 0xFE, 0xFC, 0xF8,
    0xF0, 0xE0, 0xC0, 0xFD, 0xFB, 0xF7, 0xEF, 0xDF, 0xBF, 0x55, 0xAA, 11, 17, 19, 23, 29, 37, 42,
    43, b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o',
    b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x',
];

fn probe<H>(hash: &H, reference: u64, data: &[u8], seed: u64)
where
    H: Fn(&[u8], u64) -> u64,
{
    let h = hash(data, seed);

    assert_eq!(
        h, reference,
        "hash(data = {:?}, seed = 0x{:x}) = 0x{:x}, right = 0x{:x}",
        data, seed, h, reference
    );
}

pub fn selfcheck<H>(hash: H, reference_values: &[u64])
where
    H: Fn(&[u8], u64) -> u64,
{
    let mut iter = reference_values.iter();

    probe(&hash, *iter.next().unwrap(), &[][..], 0);
    probe(&hash, *iter.next().unwrap(), &[][..], !0);
    probe(&hash, *iter.next().unwrap(), &T1HA_TEST_PATTERN[..], 0);

    for i in 1..64 {
        probe(
            &hash,
            *iter.next().unwrap(),
            &T1HA_TEST_PATTERN[..i],
            1 << i - 1,
        );
    }

    for i in 1..=7 {
        probe(
            &hash,
            *iter.next().unwrap(),
            &T1HA_TEST_PATTERN[i..64],
            !0 << i,
        );
    }

    let pattern_long = (0..512).map(|i| i as u8).collect::<Vec<_>>();
    let seed = !0 << 7;

    for i in 0..=7 {
        probe(
            &hash,
            *iter.next().unwrap(),
            &pattern_long[i..128 + i * 18],
            seed,
        );
    }
}
