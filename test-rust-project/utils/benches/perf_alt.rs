use criterion::{black_box, criterion_group, criterion_main, Criterion};
use utils::{data::*, strings};

fn benchmark_string_operations(c: &mut Criterion) {
    let test_string = "This is a test string for benchmarking string operations";

    c.bench_function("capitalize", |b| {
        b.iter(|| strings::capitalize(black_box(test_string)))
    });

    c.bench_function("reverse", |b| {
        b.iter(|| strings::reverse(black_box(test_string)))
    });

    c.bench_function("word_count", |b| {
        b.iter(|| strings::word_count(black_box(test_string)))
    });
}

fn benchmark_data_operations(c: &mut Criterion) {
    let data_points: Vec<DataPoint> = (0..1000)
        .map(|i| DataPoint::new(i, (i as f64) * 1.5, format!("Point_{}", i)))
        .collect();

    c.bench_function("process_data_points", |b| {
        b.iter(|| process_data_points(black_box(&data_points)))
    });

    c.bench_function("filter_by_threshold", |b| {
        b.iter(|| filter_by_threshold(black_box(&data_points), black_box(500.0)))
    });
}

criterion_group!(
    benches,
    benchmark_string_operations,
    benchmark_data_operations
);
criterion_main!(benches);
