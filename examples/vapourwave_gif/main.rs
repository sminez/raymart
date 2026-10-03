use gif::{Encoder, Frame, Repeat};
use rand::SeedableRng;
use raymart::{
    color,
    hit::{Hittable, HittableList},
    integrator::Integrator,
    leak_ptr,
    noise::Perlin,
    sdl::{render_with_sdl_preview, MainThreadState},
    shapes::SphereMesh,
    Backend, Bvh, Color, Rng, Scene, P3,
};
use sdl2::{event::Event, keyboard::Keycode};
use std::{
    env,
    fs::{self, File},
    hash::{DefaultHasher, Hash, Hasher},
    time::Instant,
};

const DEFAULT_SEED: &str = "deadbeef";
const SCENE: &str = "examples/vapourwave_gif/scene.yaml";

fn main() -> anyhow::Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    let seed_str = args
        .first()
        .map(|s| s.as_str())
        .unwrap_or_else(|| DEFAULT_SEED);
    let n_frames = args
        .get(1)
        .map(|s| s.parse::<usize>().unwrap())
        .unwrap_or(36);

    let preview_only = args.get(2).map(|s| s.as_str()) == Some("--preview");

    let s = fs::read_to_string(SCENE).unwrap();
    let mut lines = s.lines();
    let params: Vec<f32> = lines
        .next()
        .unwrap()
        .trim_start_matches("# ")
        .split_whitespace()
        .map(|s| s.parse::<f32>().unwrap())
        .collect();
    let mat_name = lines.next().unwrap().trim_start_matches("# ");
    let p = P3::new(params[0], params[1], params[2]);
    let r = params[3];

    // Seed rng for reproducible noise
    let mut hasher = DefaultHasher::default();
    seed_str.hash(&mut hasher);
    let seed = hasher.finish();
    let mut rng = Rng::seed_from_u64(seed);
    let noise: Perlin<256> = Perlin::new(&mut rng);

    // Init the scene
    let Scene {
        integrator,
        hittables,
        lights,
        materials,
    } = Scene::try_from_file(SCENE).unwrap();
    let lights = leak_ptr!(lights);

    // Create our iceberg
    let mat = materials
        .get(mat_name)
        .expect("unknown material name specified for sphere");
    let berg = SphereMesh::noise_sphere_with_source(p, r, 7, false, 2.0, 9, *mat, &noise);

    let (w, h) = integrator.camera().image_dims();
    let (mut mts, canvas) = MainThreadState::init(w as u32, h as u32)?;
    let tc = canvas.texture_creator();
    let mut backend = Backend::init(w as u32, h as u32, canvas, &tc)?;

    if preview_only {
        render_frame(
            0,
            n_frames,
            berg,
            hittables.clone(),
            lights,
            integrator,
            &mut mts,
            &mut backend,
        );
    } else {
        render_gif(
            n_frames,
            berg,
            hittables.clone(),
            lights,
            integrator,
            &mut mts,
            &mut backend,
        );
    }

    loop {
        match mts.wait_event() {
            Event::Quit { .. } => return Ok(()),
            Event::KeyDown {
                keycode: Some(Keycode::Q | Keycode::Escape),
                repeat: false,
                ..
            } => return Ok(()),
            _ => (),
        }
    }
}

#[expect(clippy::too_many_arguments)]
fn render_frame(
    i: usize,
    n_frames: usize,
    mut berg: SphereMesh,
    mut hittables: Vec<&'static dyn Hittable>,
    lights: &'static HittableList,
    integrator: &mut dyn Integrator,
    mts: &mut MainThreadState,
    backend: &mut Backend<'_>,
) -> (bool, Vec<Color>) {
    berg.rotate_vertices_y(i as f32 * 360.0 / n_frames as f32);
    hittables.push(berg.into_dyn_hittable());

    eprintln!("\nComputing bvh tree...");
    let bvh = Bvh::new(hittables);

    eprintln!("Rendering frame {i}...");
    render_with_sdl_preview(integrator, bvh, lights, mts, backend)
}

fn render_gif(
    n_frames: usize,
    berg: SphereMesh,
    hittables: Vec<&'static dyn Hittable>,
    lights: &'static HittableList,
    integrator: &mut dyn Integrator,
    mts: &mut MainThreadState,
    backend: &mut Backend<'_>,
) {
    let mut frames = Vec::with_capacity(n_frames);
    let (w, h) = integrator.camera().image_dims();

    let start = Instant::now();

    for i in 0..n_frames {
        let (early_return, raw_frame) = render_frame(
            i,
            n_frames,
            berg.clone(),
            hittables.clone(),
            lights,
            integrator,
            mts,
            backend,
        );
        let frame: Vec<u8> = raw_frame.into_iter().flat_map(color::to_rgb).collect();
        frames.push(frame);

        if early_return {
            break;
        }
    }

    eprintln!("\nRendering GIF");

    let mut f = File::create("out.gif").unwrap();
    let mut encoder = Encoder::new(&mut f, w as u16, h as u16, &[]).unwrap();
    encoder.set_repeat(Repeat::Infinite).unwrap();

    for pixels in frames.iter() {
        let frame = Frame::from_rgb(w as u16, h as u16, pixels);
        encoder.write_frame(&frame).unwrap();
    }

    eprintln!("\nDone");
    let render_time = Instant::now().duration_since(start);
    eprintln!("Total render time: {}s", render_time.as_secs());
}
