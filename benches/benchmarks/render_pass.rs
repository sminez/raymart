use criterion::{criterion_group, measurement::WallTime, BenchmarkGroup, Criterion};
use raymart::{leak_ptr, Bvh, Color, Scene};

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Render pass");

    _ = rayon::ThreadPoolBuilder::new()
        .num_threads(std::thread::available_parallelism().unwrap().get())
        .build_global();

    scene_render_pass(
        "cuboids cornell",
        include_str!("./cuboids_cornell.yaml"),
        &mut group,
    );
    scene_render_pass(
        "glass ball cornell",
        include_str!("./glass_ball_cornell.yaml"),
        &mut group,
    );
    scene_render_pass(
        "dragon head",
        include_str!("./dragon_head.yaml"),
        &mut group,
    );

    group.finish();
}

fn scene_render_pass(title: &str, scene: &str, group: &mut BenchmarkGroup<'_, WallTime>) {
    let Scene {
        integrator,
        hittables,
        lights,
        ..
    } = Scene::try_from_str(scene).unwrap();

    let bvh = Bvh::new(hittables);
    let lights = leak_ptr!(lights);

    let (w, h) = integrator.camera().image_dims();
    let pixels = vec![Color::default(); w * h];
    let mut new_pixels = pixels.clone();

    group.bench_function(title, |b| {
        b.iter(|| {
            integrator.next_render_pass(1, &bvh, lights, &pixels, &mut new_pixels);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
