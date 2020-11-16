extern crate seahash;
extern crate core;
extern crate criterion;

use core::hash::Hasher;
use criterion::{criterion_group, criterion_main, Criterion};

fn gigabyte(c: &mut Criterion) {
    c.bench_function("gigabyte", |b| b.iter(|| {
        let mut x = 0;
        let mut buf = [15; 4096];

        for _ in 0..250000 {
            x ^= seahash::hash(&buf);
            buf[0] += buf[0].wrapping_add(1);
        }

        x
    }));

    c.bench_function("gigabyte_stream", |b| b.iter(|| {
        let mut buf = [15;4096];
        let mut hasher = seahash::SeaHasher::default();

        for _ in 0..250_000 {
            Hasher::write(&mut hasher,&buf);
            buf[0] += buf[0].wrapping_add(1);
        }
        hasher.finish()
    }));
}

criterion_group!(benches, gigabyte);
criterion_main!(benches);
