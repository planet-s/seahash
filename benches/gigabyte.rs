extern crate seahash;
extern crate core;
extern crate criterion;

use core::hash::Hasher;
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput, BenchmarkId};

fn gigabyte(c: &mut Criterion) {
    let buf = vec![15; 4 * 4096];
    let sizes = [8, 64, 1024, 4096, 4 * 4096];

    let mut group = c.benchmark_group("buffer");

    for size in &sizes {
        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                black_box(seahash::hash(&buf[..size]));
            })
        });
    }

    group.finish();

    let mut group = c.benchmark_group("stream");

    for size in &sizes {
        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_with_setup(
                || seahash::SeaHasher::default(),
                |mut h: seahash::SeaHasher| {
                    // use chunks of 32 bytes to simulate some looping on a single hasher value
                    for _ in 0..size/32 {
                        h.write(&buf[..32]);
                    }
                    // this will mostly be an empty slice, but that is a possible Hasher api usage
                    h.write(&buf[..(size % 32)]);
                    black_box(h.finish())
            })
        });
    }

    group.finish();
}

criterion_group!(benches, gigabyte);
criterion_main!(benches);
