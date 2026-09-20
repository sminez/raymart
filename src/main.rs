use raymart::{sdl::MainThreadState, Backend, Bvh, Scene, SCENE_PATH};
use sdl2::{event::Event, keyboard::Keycode};
use std::env;

fn main() -> anyhow::Result<()> {
    let path = env::args().nth(1).unwrap_or_else(|| SCENE_PATH.to_string());
    eprintln!("scene = {path}");

    let s = Scene::try_from_file(&path).unwrap_or_default();
    let (hittables, camera) = s.load_scene();

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
    camera.render_sdl(bvh_tree, &mut mts, &mut backend);

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
