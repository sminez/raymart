pub mod bvh;
pub mod color;
pub mod hit;
pub mod material;
pub mod noise;
pub mod ray;
pub mod scene;
pub mod sdl;
pub mod v3;

pub use bvh::Bvh;
pub use color::Color;
pub use hit::HitRecord;
pub use ray::Ray;
pub use scene::Scene;
pub use sdl::Backend;
pub use v3::{P3, V3};

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
