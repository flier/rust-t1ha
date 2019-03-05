#[macro_use]
extern crate criterion;

use criterion::{Criterion, ParameterizedBenchmark, Throughput};

use t1ha::t1ha0;

const KB: usize = 1024;

fn bench_t1ha0(c: &mut Criterion) {
    let seed = 0x0123456789ABCDEF;

    c.bench(
        "t1ha0",
        ParameterizedBenchmark::new(
            "t1ha0",
            move |b, &&size| {
                let data = (0..size).map(|b| b as u8).collect::<Vec<_>>();

                b.iter(|| t1ha0(&data[..], seed));
            },
            &[8, 32, 64, 256, 512, KB, 2 * KB, 4 * KB, 8 * KB],
        )
        .throughput(|&&elems| Throughput::Bytes(elems as u32)),
    );
}

criterion_group!(benches, bench_t1ha0);
criterion_main!(benches);
