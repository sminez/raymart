use criterion::criterion_main;

mod benchmarks;

criterion_main! {
    benchmarks::render_pass::benches,
}
