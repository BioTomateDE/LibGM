use std::hint::black_box;
use std::path::Path;
use criterion::{criterion_group, criterion_main, Criterion};
use libgm::{build_data_file, parse_data_file};


fn builder_benchmark(c: &mut Criterion) {
    biologischer_log::init(env!("CARGO_PKG_NAME"));
    
    let data_path = Path::new("data.win");
    let raw_data = std::fs::read(data_path).expect("could not read data file");
    let gm_data = parse_data_file(&raw_data, false).expect("could not parse data file");
    drop(raw_data);
    
    c.bench_function("serialize", |b| {
        b.iter(|| {
            build_data_file(black_box(&gm_data)).expect("could not build data file");
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(std::time::Duration::from_secs(4))
        .measurement_time(std::time::Duration::from_secs(20))
        .sample_size(30);
    targets = builder_benchmark
}
criterion_main!(benches);
