use raymart::{
    color, leak_ptr,
    sdl::{render_with_sdl_preview, MainThreadState},
    Backend, Bvh, Scene,
};
use sdl2::{event::Event, keyboard::Keycode};
use std::{env, fs};

const SCENE_PATH: &str = "scene.yaml";

fn main() -> anyhow::Result<()> {
    let path = env::args().nth(1).unwrap_or_else(|| SCENE_PATH.to_string());
    eprintln!("scene = {path}");

    let Scene {
        integrator,
        hittables,
        lights,
        ..
    } = Scene::try_from_file(&path)?;

    eprintln!("{integrator:#?}");
    let lights = leak_ptr!(lights);

    eprintln!("Computing bvh tree...");
    let bvh = Bvh::new(hittables);
    eprintln!(
        "BVH bounding box:\n  x={:?}\n  y={:?}\n  z={:?}",
        bvh.bbox.x, bvh.bbox.y, bvh.bbox.z,
    );

    let (w, h) = integrator.camera().image_dims();
    let (mut mts, canvas) = MainThreadState::init(w as u32, h as u32)?;
    let tc = canvas.texture_creator();
    let mut backend = Backend::init(w as u32, h as u32, canvas, &tc)?;

    eprintln!("Rendering...");
    let (_, pixels) = render_with_sdl_preview(integrator, bvh, lights, &mut mts, &mut backend);

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
