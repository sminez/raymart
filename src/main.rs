use rand::SeedableRng;
use raymart::{color, leak_ptr, sdl::MainThreadState, Backend, Bvh, Rng, Scene, SCENE_PATH};
use sdl2::{event::Event, keyboard::Keycode};
use std::{env, fs};

fn main() -> anyhow::Result<()> {
    let path = env::args().nth(1).unwrap_or_else(|| SCENE_PATH.to_string());
    eprintln!("scene = {path}");

    let s = Scene::try_from_file(&path).unwrap_or_default();
    let mut rng = Rng::seed_from_u64(0);
    let (hittables, lights, camera) = s.load_scene(&mut rng);
    let lights = leak_ptr!(lights);

    eprintln!("Computing bvh tree...");
    let bvh_tree = Bvh::new(hittables);
    eprintln!(
        "BVH bounding box:\n  x={:?}\n  y={:?}\n  z={:?}",
        bvh_tree.bbox.x, bvh_tree.bbox.y, bvh_tree.bbox.z,
    );

    let (w, h) = camera.dims();
    let (mut mts, canvas) = MainThreadState::init(w, h)?;
    let tc = canvas.texture_creator();
    let mut backend = Backend::init(w, h, canvas, &tc)?;

    eprintln!("Rendering...");
    let (_, pixels) = camera.render_sdl(bvh_tree, lights, &mut mts, &mut backend);

    eprintln!("writing ppm file");
    let s: String = pixels.iter().map(|c| color::ppm_string(*c)).collect();
    fs::write("test.ppm", format!("P3\n{w} {h}\n255\n{s}")).unwrap();

    eprintln!("\nDone");

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
