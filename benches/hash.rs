#![allow(deprecated)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate criterion;
#[macro_use]
extern crate cfg_if;

use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::hash::SipHasher;
use std::io::BufReader;
use std::mem;
use std::slice;

use criterion::{black_box, BenchmarkId, Criterion, Throughput};

use ahash::AHasher;
use farmhash::{hash32_with_seed as farmhash32, hash64_with_seed as farmhash64};
use fnv::FnvHasher;
use fxhash::{hash32 as fxhash32, hash64 as fxhash64};
use meowhash::{MeowHash, MeowHasher};
use metrohash::{MetroHash128, MetroHash64};
use murmur3::{murmur3_32, murmur3_x64_128, murmur3_x86_128};
use rustc_hash::FxHasher;
use seahash::hash_seeded as seahash64;
use t1ha::{t1ha0_32, t1ha1, t1ha2_atonce, t1ha2_atonce128};
use twox_hash::{XxHash as XxHash64, XxHash32};
use wyhash::wyhash;
use xxhash2::{hash32 as xxhash32, hash64 as xxhash64};

cfg_if! {
    if #[cfg(target_feature = "aes")] {
        use t1ha::t1ha0_ia32aes_noavx;
    } else {
        fn t1ha0_ia32aes_noavx(_data: &[u8], _seed: u64) -> u64 {
            0
        }
    }
}

cfg_if! {
    if #[cfg(target_feature = "avx")] {
        use t1ha::t1ha0_ia32aes_avx;
    } else {
        fn t1ha0_ia32aes_avx(_data: &[u8], _seed: u64) -> u64 {
            0
        }
    }
}

cfg_if! {
    if #[cfg(target_feature = "avx2")] {
        use t1ha::t1ha0_ia32aes_avx2;
    } else {
        fn t1ha0_ia32aes_avx2(_data: &[u8], _seed: u64) -> u64 {
            0
        }
    }
}

const KB: u64 = 1024;
const SEED: u64 = 0x0123456789ABCDEF;
const PARAMS: [u64; 7] = [7, 8, 32, 256, KB, 4 * KB, 16 * KB];

lazy_static! {
    static ref DATA: Vec<u8> = (0..16 * KB).map(|b| b as u8).collect::<Vec<_>>();
}

fn bench_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory");

    for size in PARAMS {
        group.throughput(Throughput::Bytes(size)).bench_with_input(
            BenchmarkId::from_parameter(size),
            &size,
            |b, _| {
                let s = unsafe {
                    slice::from_raw_parts(
                        DATA.as_ptr().cast::<u32>(),
                        size as usize / mem::size_of::<u32>(),
                    )
                };

                b.iter(|| {
                    black_box(s.iter().fold(0u64, |acc, &x| acc + x as u64));
                })
            },
        );
    }
}

fn bench_hash32(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash32");

    for size in PARAMS {
        let data = &DATA[..size as usize];

        group
            .throughput(Throughput::Bytes(size))
            .bench_with_input(BenchmarkId::new("t1ha0_32", size), &size, |b, _| {
                b.iter(|| t1ha0_32(data, SEED));
            })
            .bench_with_input(BenchmarkId::new("murmur3_32", size), &size, |b, _| {
                b.iter(|| murmur3_32(&mut BufReader::new(data), SEED as u32));
            })
            .bench_with_input(BenchmarkId::new("farmhash32", size), &size, |b, _| {
                b.iter(|| farmhash32(data, SEED as u32));
            })
            .bench_with_input(BenchmarkId::new("xxhash32", size), &size, |b, _| {
                b.iter(|| xxhash32(data, SEED as u32));
            })
            .bench_with_input(
                BenchmarkId::new("twox_hash::XxHash32", size),
                &size,
                |b, _| {
                    b.iter(|| {
                        let mut h = XxHash32::with_seed(SEED as u32);
                        h.write(data);
                        h.finish()
                    });
                },
            )
            .bench_with_input(BenchmarkId::new("fxhash32", size), &size, |b, _| {
                b.iter(|| fxhash32(data));
            });
    }
}

fn bench_hash64(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash64");

    for size in PARAMS {
        let data = &DATA[..size as usize];

        group
            .throughput(Throughput::Bytes(size))
            .bench_with_input(BenchmarkId::new("t1ha1", size), &size, |b, _| {
                b.iter(|| t1ha1(data, SEED));
            })
            .bench_with_input(BenchmarkId::new("t1ha2_atonce", size), &size, |b, _| {
                b.iter(|| t1ha2_atonce(data, SEED));
            });

        if cfg!(target_feature = "aes") {
            group.bench_with_input(
                BenchmarkId::new("t1ha0_ia32aes_noavx", size),
                &size,
                |b, _| b.iter(|| t1ha0_ia32aes_noavx(data, SEED)),
            );
        }

        if cfg!(target_feature = "avx") {
            group.bench_with_input(
                BenchmarkId::new("t1ha0_ia32aes_avx", size),
                &size,
                |b, _| b.iter(|| t1ha0_ia32aes_avx(data, SEED)),
            );
        }

        if cfg!(target_feature = "avx2") {
            group.bench_with_input(
                BenchmarkId::new("t1ha0_ia32aes_avx2", size),
                &size,
                |b, _| b.iter(|| t1ha0_ia32aes_avx2(data, SEED)),
            );
        }

        group
            .bench_with_input(
                BenchmarkId::new("hash_map::DefaultHasher", size),
                &size,
                |b, _| {
                    b.iter(|| {
                        let mut h = DefaultHasher::new();
                        h.write(data);
                        h.finish()
                    });
                },
            )
            .bench_with_input(BenchmarkId::new("siphash", size), &size, |b, _| {
                b.iter(|| {
                    let mut h = SipHasher::new_with_keys(SEED, SEED);
                    h.write(data);
                    h.finish()
                });
            })
            .bench_with_input(BenchmarkId::new("metrohash64", size), &size, |b, _| {
                b.iter(|| {
                    let mut h = MetroHash64::with_seed(SEED);
                    h.write(data);
                    h.finish()
                });
            })
            .bench_with_input(BenchmarkId::new("farmhash64", size), &size, |b, _| {
                b.iter(|| farmhash64(data, SEED));
            })
            .bench_with_input(BenchmarkId::new("fnv64", size), &size, |b, _| {
                b.iter(|| {
                    let mut h = FnvHasher::with_key(SEED);
                    h.write(data);
                    h.finish()
                });
            })
            .bench_with_input(BenchmarkId::new("xxhash64", size), &size, |b, _| {
                b.iter(|| xxhash64(data, SEED));
            })
            .bench_with_input(
                BenchmarkId::new("twox_hash::XxHash", size),
                &size,
                |b, _| {
                    b.iter(|| {
                        let mut h = XxHash64::with_seed(SEED);
                        h.write(data);
                        h.finish()
                    });
                },
            )
            .bench_with_input(BenchmarkId::new("seahash", size), &size, |b, _| {
                b.iter(|| seahash64(data, SEED, SEED, SEED, SEED));
            })
            .bench_with_input(BenchmarkId::new("fxhash64", size), &size, |b, _| {
                b.iter(|| fxhash64(data));
            })
            .bench_with_input(BenchmarkId::new("ahash", size), &size, |b, _| {
                b.iter(|| {
                    let mut h = AHasher::default();
                    h.write(data);
                    h.finish()
                });
            })
            .bench_with_input(
                BenchmarkId::new("rustc_hash::FxHasher", size),
                &size,
                |b, _| {
                    b.iter(|| {
                        let mut h = FxHasher::default();
                        h.write(data);
                        h.finish()
                    });
                },
            )
            .bench_with_input(BenchmarkId::new("wyhash", size), &size, |b, _| {
                b.iter(|| wyhash(data, SEED));
            });
    }
}

fn bench_hash128(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash128");

    for size in PARAMS {
        let data = &DATA[..size as usize];

        group
            .throughput(Throughput::Bytes(size))
            .bench_with_input(BenchmarkId::new("t1ha2_atonce128", size), &size, |b, _| {
                b.iter(|| t1ha2_atonce128(data, SEED))
            })
            .bench_with_input(BenchmarkId::new("metrohash128", size), &size, |b, _| {
                b.iter(|| {
                    let mut h = MetroHash128::with_seed(SEED);
                    h.write(data);
                    h.finish128()
                })
            });

        if cfg!(target_arch = "x86_64") {
            group.bench_with_input(BenchmarkId::new("murmur3_x64_128", size), &size, |b, _| {
                b.iter(|| murmur3_x64_128(&mut BufReader::new(data), SEED as _))
            });
        }

        if cfg!(target_arch = "x86") {
            group.bench_with_input(BenchmarkId::new("murmur3_x86_128", size), &size, |b, _| {
                b.iter(|| murmur3_x86_128(&mut BufReader::new(data), SEED as _))
            });
        }

        if cfg!(target_feature = "aes") {
            group.bench_with_input(BenchmarkId::new("meowhash128", size), &size, |b, _| {
                b.iter(|| {
                    MeowHasher::digest_with_seed(MeowHash::expand_seed(&SEED.to_ne_bytes()), data)
                });
            });
        }
    }
}

criterion_group!(
    benches,
    bench_memory,
    bench_hash32,
    bench_hash64,
    bench_hash128
);
criterion_main!(benches);
