#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate criterion;

use std::mem;
use std::slice;

use criterion::{black_box, Criterion, ParameterizedBenchmark, Throughput};

use t1ha::{t1ha0, t1ha0_ia32aes_avx, t1ha0_ia32aes_avx2};

const KB: usize = 1024;
const SEED: u64 = 0x0123456789ABCDEF;
const PARAMS: [usize; 11] = [7, 8, 32, 64, 256, 512, KB, 2 * KB, 4 * KB, 8 * KB, 16 * KB];

lazy_static! {
    static ref DATA: Vec<u8> = (0..16 * KB).map(|b| b as u8).collect::<Vec<_>>();
}

fn bench_t1ha0(c: &mut Criterion) {
    c.bench(
        "memory scan",
        ParameterizedBenchmark::new(
            "sum",
            move |b, &&size| {
                let s = unsafe {
                    slice::from_raw_parts(DATA.as_ptr() as *mut u32, size / mem::size_of::<u32>())
                };

                b.iter(|| {
                    black_box(s.iter().fold(0u64, |acc, &x| acc + x as u64));
                })
            },
            &PARAMS,
        )
        .throughput(|&&size| Throughput::Bytes(size as u32)),
    );

    c.bench(
        "t1ha0",
        ParameterizedBenchmark::new(
            "t1ha0",
            move |b, &&size| {
                b.iter(|| t1ha0(&DATA[..size], SEED));
            },
            &PARAMS,
        )
        .with_function("t1ha0_ia32aes_avx", move |b, &&size| {
            b.iter(|| t1ha0_ia32aes_avx(&DATA[..size], SEED));
        })
        .with_function("t1ha0_ia32aes_avx2", move |b, &&size| {
            b.iter(|| t1ha0_ia32aes_avx2(&DATA[..size], SEED));
        })
        .throughput(|&&size| Throughput::Bytes(size as u32)),
    );
}

criterion_group!(benches, bench_t1ha0);
criterion_main!(benches);
