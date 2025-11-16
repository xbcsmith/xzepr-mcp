//! Benchmark suite for XZepr MCP
//! Implementation will be added in Phase 3

use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_stub(c: &mut Criterion) {
    c.bench_function("stub", |b| b.iter(|| 1 + 1));
}

criterion_group!(benches, benchmark_stub);
criterion_main!(benches);
