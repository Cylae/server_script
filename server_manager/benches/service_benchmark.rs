use criterion::{black_box, criterion_group, criterion_main, Criterion};
use server_manager::services::get_all_services;

fn benchmark_get_all_services(c: &mut Criterion) {
    c.bench_function("get_all_services", |b| b.iter(|| {
        black_box(get_all_services());
    }));
}

criterion_group!(benches, benchmark_get_all_services);
criterion_main!(benches);
