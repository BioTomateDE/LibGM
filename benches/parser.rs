use std::hint::black_box;
use std::path::Path;
use criterion::{criterion_group, criterion_main, Criterion};
use libgm::parse_data_file;


fn parser_benchmark(c: &mut Criterion) {
    biologischer_log::init(env!("CARGO_PKG_NAME"));
    let data_path = Path::new("data.win");
    let data = std::fs::read(data_path).expect("could not read data file");
    c.bench_function("deserialize", |b| {
        b.iter(|| {
            parse_data_file(black_box(&data), false).expect("could not parse data file");
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(std::time::Duration::from_secs(4))
        .measurement_time(std::time::Duration::from_secs(20))
        .sample_size(30);
    targets = parser_benchmark
}
criterion_main!(benches);
