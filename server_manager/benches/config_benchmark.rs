use criterion::{criterion_group, criterion_main, Criterion};
use server_manager::core::config::Config;
use tokio::runtime::Runtime;

fn benchmark_config_load_async(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    // Ensure config.yaml exists
    std::fs::write("config.yaml", "disabled_services: []").unwrap();

    c.bench_function("config_load_async", |b| {
        b.to_async(&rt).iter(|| async {
            let _ = Config::load_async().await;
        })
    });
}

criterion_group!(benches, benchmark_config_load_async);
criterion_main!(benches);
