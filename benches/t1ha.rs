#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate criterion;
#[macro_use]
extern crate cfg_if;

use std::mem;
use std::slice;

use criterion::{black_box, BenchmarkId, Criterion, Throughput};

use t1ha::{t1ha0_32, t1ha1, t1ha2_atonce, t1ha2_atonce128, T1ha2Hasher};

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
const PARAMS: [u64; 11] = [7, 8, 32, 64, 256, 512, KB, 2 * KB, 4 * KB, 8 * KB, 16 * KB];

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

fn bench_t1ha0(c: &mut Criterion) {
    let mut group = c.benchmark_group("t1ha0");

    for size in PARAMS {
        let data = &DATA[..size as usize];

        group.throughput(Throughput::Bytes(size)).bench_with_input(
            BenchmarkId::new("t1ha0_32", size),
            &size,
            |b, _| {
                b.iter(|| t1ha0_32(data, SEED));
            },
        );

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
    }
}

fn bench_t1ha1(c: &mut Criterion) {
    let mut group = c.benchmark_group("t1ha1");

    for size in PARAMS {
        let data = &DATA[..size as usize];

        group.throughput(Throughput::Bytes(size)).bench_with_input(
            BenchmarkId::new("t1ha1", size),
            &size,
            |b, _| b.iter(|| t1ha1(data, SEED)),
        );
    }
}

fn bench_t1ha2(c: &mut Criterion) {
    let mut group = c.benchmark_group("t1ha2");

    for size in PARAMS {
        let data = &DATA[..size as usize];

        group
            .throughput(Throughput::Bytes(size))
            .bench_with_input(BenchmarkId::new("t1ha2_atonce", size), &size, |b, _| {
                b.iter(|| t1ha2_atonce(data, SEED))
            })
            .bench_with_input(BenchmarkId::new("t1ha2_stream", size), &size, |b, _| {
                b.iter(|| {
                    let mut h = T1ha2Hasher::with_seeds(SEED, SEED);
                    h.update(data);
                    h.finish()
                })
            })
            .bench_with_input(BenchmarkId::new("t1ha2_atonce128", size), &size, |b, _| {
                b.iter(|| t1ha2_atonce128(data, SEED))
            })
            .bench_with_input(BenchmarkId::new("t1ha2_stream123", size), &size, |b, _| {
                b.iter(|| {
                    let mut h = T1ha2Hasher::with_seeds(SEED, SEED);
                    h.update(data);
                    h.finish128()
                })
            });
    }
}

criterion_group!(benches, bench_memory, bench_t1ha0, bench_t1ha1, bench_t1ha2);
criterion_main!(benches);
