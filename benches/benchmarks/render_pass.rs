use criterion::{criterion_group, measurement::WallTime, BenchmarkGroup, Criterion};
use raymart::{Bvh, Color, Scene};

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Render pass");

    scene_render_pass(
        "cuboids cornell",
        include_str!("./cuboids_cornell.toml"),
        &mut group,
    );
    scene_render_pass(
        "glass ball cornell",
        include_str!("./glass_ball_cornell.toml"),
        &mut group,
    );
    scene_render_pass(
        "dragon head",
        include_str!("./dragon_head.toml"),
        &mut group,
    );

    group.finish();
}

fn scene_render_pass(title: &str, scene: &str, group: &mut BenchmarkGroup<'_, WallTime>) {
    let s = Scene::try_from_str(scene).unwrap();
    let (hittables, camera) = s.load_scene();
    let bvh = Bvh::new(hittables);
    let (w, h) = camera.dims();

    let mut pixels = vec![Color::default(); w as usize * h as usize];

    group.bench_function(title, |b| {
        b.iter(|| {
            camera.render_pass(&bvh, &mut pixels);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
