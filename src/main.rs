pub mod bvh;
pub mod color;
pub mod hit;
pub mod material;
pub mod noise;
pub mod ray;
pub mod scene;
pub mod v3;

use std::env;

use bvh::Bvh;
use color::Color;
use hit::HitRecord;
use ray::Ray;
use scene::Scene;
use v3::{P3, V3};

pub const BG_COLOR: Color = Color::new(0.7, 0.8, 1.0); // default scene background color
pub const ASPECT_RATIO: f32 = 16.0 / 10.0; // image aspect ratio
pub const IMAGE_WIDTH: u16 = 1000; // image width in pixels
pub const SAMPLES_PER_PIXEL: u16 = 4500; // number of random samples per pixel
pub const STEP_SIZE: u16 = 100; // number of samples per render step
pub const DEBUG_SAMPLES_PER_PIXEL: u16 = 10; // number of random samples per pixel
pub const MAX_BOUNCES: u8 = 50; // maximum number of ray bounces allowed
pub const SCENE_PATH: &str = "scene.toml";

#[macro_export]
macro_rules! p {
    ($x:expr, $y:expr, $z:expr) => {
        P3::new($x as f32, $y as f32, $z as f32)
    };
}

#[macro_export]
macro_rules! v {
    ($x:expr, $y:expr, $z:expr) => {
        V3::new($x as f32, $y as f32, $z as f32)
    };
}

fn main() {
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

    eprintln!("Rendering...");
    camera.render_ppm(bvh_tree);

    eprintln!("\nDone");
}
