#![allow(deprecated)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate criterion;

use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::hash::SipHasher;
use std::io::BufReader;

use criterion::{Criterion, ParameterizedBenchmark, Throughput};

use ahash::AHasher;
use farmhash::{hash32_with_seed as farmhash32, hash64_with_seed as farmhash64};
use fnv::FnvHasher;
use fxhash::{hash32 as fxhash32, hash64 as fxhash64};
use meowhash::MeowHasher;
use metrohash::{MetroHash128, MetroHash64};
use murmur3::{murmur3_32, murmur3_x64_128, murmur3_x86_128};
use rustc_hash::FxHasher;
use seahash::hash_seeded as seahash64;
use t1ha::{t1ha0_32, t1ha1, t1ha2_atonce, t1ha2_atonce128};
use twox_hash::{XxHash as XxHash64, XxHash32};
use xxhash2::{hash32 as xxhash32, hash64 as xxhash64};

#[cfg(target_feature = "aes")]
use t1ha::t1ha0_ia32aes_noavx;
#[cfg(not(target_feature = "aes"))]
fn t1ha0_ia32aes_noavx(_data: &[u8], _seed: u64) -> u64 {
    0
}

#[cfg(target_feature = "avx2")]
use t1ha::t1ha0_ia32aes_avx2;
#[cfg(not(target_feature = "avx2"))]
fn t1ha0_ia32aes_avx2(_data: &[u8], _seed: u64) -> u64 {
    0
}

#[cfg(target_feature = "avx")]
use t1ha::t1ha0_ia32aes_avx;
#[cfg(not(target_feature = "avx"))]
fn t1ha0_ia32aes_avx(_data: &[u8], _seed: u64) -> u64 {
    0
}

const KB: usize = 1024;
const SEED: u64 = 0x0123456789ABCDEF;
const PARAMS: [usize; 7] = [7, 8, 32, 256, KB, 4 * KB, 16 * KB];

lazy_static! {
    static ref DATA: Vec<u8> = (0..16 * KB).map(|b| b as u8).collect::<Vec<_>>();
}

fn bench_hash32(c: &mut Criterion) {
    c.bench(
        "hash32",
        ParameterizedBenchmark::new(
            "t1ha0_32",
            move |b, &&size| {
                b.iter(|| t1ha0_32(&DATA[..size], SEED));
            },
            &PARAMS,
        )
        .with_function("murmur3_32", move |b, &&size| {
            b.iter(|| {
                let mut r = BufReader::new(&DATA[..size]);

                murmur3_32(&mut r, SEED as u32)
            });
        })
        .with_function("farmhash32", move |b, &&size| {
            b.iter(|| farmhash32(&DATA[..size], SEED as u32));
        })
        .with_function("xxhash32", move |b, &&size| {
            b.iter(|| xxhash32(&DATA[..size], SEED as u32));
        })
        .with_function("twox_hash::XxHash32", move |b, &&size| {
            b.iter(|| {
                let mut h = XxHash32::with_seed(SEED as u32);
                h.write(&DATA[..size]);
                h.finish()
            });
        })
        .with_function("fxhash32", move |b, &&size| {
            b.iter(|| fxhash32(&DATA[..size]));
        })
        .throughput(|&&size| Throughput::Bytes(size as u32)),
    );
}

fn bench_hash64(c: &mut Criterion) {
    let mut bench = ParameterizedBenchmark::new(
        "t1ha1",
        move |b, &&size| {
            b.iter(|| t1ha1(&DATA[..size], SEED));
        },
        &PARAMS,
    )
    .with_function("t1ha2_atonce", move |b, &&size| {
        b.iter(|| t1ha2_atonce(&DATA[..size], SEED));
    });

    if cfg!(target_feature = "aes") {
        bench = bench.with_function("t1ha0_ia32aes_noavx", move |b, &&size| {
            b.iter(|| t1ha0_ia32aes_noavx(&DATA[..size], SEED));
        });
    }
    if cfg!(target_feature = "avx") {
        bench = bench.with_function("t1ha0_ia32aes_avx", move |b, &&size| {
            b.iter(|| t1ha0_ia32aes_avx(&DATA[..size], SEED));
        });
    }
    if cfg!(target_feature = "avx2") {
        bench = bench.with_function("t1ha0_ia32aes_avx2", move |b, &&size| {
            b.iter(|| t1ha0_ia32aes_avx2(&DATA[..size], SEED));
        });
    }

    c.bench(
        "hash64",
        bench
            .with_function("hash_map::DefaultHasher", move |b, &&size| {
                b.iter(|| {
                    let mut h = DefaultHasher::new();
                    h.write(&DATA[..size]);
                    h.finish()
                });
            })
            .with_function("siphash", move |b, &&size| {
                b.iter(|| {
                    let mut h = SipHasher::new_with_keys(SEED, SEED);
                    h.write(&DATA[..size]);
                    h.finish()
                });
            })
            .with_function("metrohash64", move |b, &&size| {
                b.iter(|| {
                    let mut h = MetroHash64::with_seed(SEED);
                    h.write(&DATA[..size]);
                    h.finish()
                });
            })
            .with_function("farmhash64", move |b, &&size| {
                b.iter(|| farmhash64(&DATA[..size], SEED));
            })
            .with_function("fnv64", move |b, &&size| {
                b.iter(|| {
                    let mut h = FnvHasher::with_key(SEED);
                    h.write(&DATA[..size]);
                    h.finish()
                });
            })
            .with_function("xxhash64", move |b, &&size| {
                b.iter(|| xxhash64(&DATA[..size], SEED));
            })
            .with_function("twox_hash::XxHash", move |b, &&size| {
                b.iter(|| {
                    let mut h = XxHash64::with_seed(SEED);
                    h.write(&DATA[..size]);
                    h.finish()
                });
            })
            .with_function("seahash", move |b, &&size| {
                b.iter(|| seahash64(&DATA[..size], SEED, SEED, SEED, SEED));
            })
            .with_function("fxhash64", move |b, &&size| {
                b.iter(|| fxhash64(&DATA[..size]));
            })
            .with_function("ahash", move |b, &&size| {
                b.iter(|| {
                    let mut h = AHasher::new_with_keys(SEED, SEED);
                    h.write(&DATA[..size]);
                    h.finish()
                });
            })
            .with_function("rustc_hash::FxHasher", move |b, &&size| {
                b.iter(|| {
                    let mut h = FxHasher::default();
                    h.write(&DATA[..size]);
                    h.finish()
                });
            })
            .throughput(|&&size| Throughput::Bytes(size as u32)),
    );
}

fn bench_hash128(c: &mut Criterion) {
    let mut bench = ParameterizedBenchmark::new(
        "t1ha2_atonce128",
        move |b, &&size| {
            b.iter(|| t1ha2_atonce128(&DATA[..size], SEED));
        },
        &PARAMS,
    )
    .with_function("metrohash128", move |b, &&size| {
        b.iter(|| {
            let mut h = MetroHash128::with_seed(SEED);
            h.write(&DATA[..size]);
            h.finish128()
        });
    });

    if cfg!(target_arch = "x86_64") {
        bench = bench.with_function("murmur3_x64_128", move |b, &&size| {
            b.iter(|| {
                let mut r = BufReader::new(&DATA[..size]);
                let mut out = [0; 16];

                murmur3_x64_128(&mut r, SEED as u32, &mut out);
            });
        });
    }

    if cfg!(target_arch = "x86") {
        bench = bench.with_function("murmur3_x86_128", move |b, &&size| {
            b.iter(|| {
                let mut r = BufReader::new(&DATA[..size]);
                let mut out = [0; 16];

                murmur3_x86_128(&mut r, SEED as u32, &mut out);
            });
        });
    }

    if cfg!(target_feature = "aes") {
        bench = bench.with_function("meowhash128", move |b, &&size| {
            b.iter(|| MeowHasher::digest_with_seed(SEED as u128, &DATA[..size]));
        });
    }

    c.bench(
        "hash128",
        bench.throughput(|&&size| Throughput::Bytes(size as u32)),
    );
}

criterion_group!(benches, bench_hash32, bench_hash64, bench_hash128);
criterion_main!(benches);
