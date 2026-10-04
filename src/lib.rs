pub use rand_xoshiro::Xoshiro128Plus as Rng;

pub mod bvh;
pub mod camera;
pub mod color;
pub mod gpu;
pub mod hit;
pub mod integrator;
pub mod material;
pub mod noise;
pub mod pdf;
pub mod ray;
pub mod sampler;
pub mod scene;
pub mod sdl;
pub mod shapes;
pub mod texture;
pub mod v3;

pub use bvh::Bvh;
pub use color::Color;
pub use hit::HitRecord;
pub use ray::Ray;
pub use scene::Scene;
pub use sdl::Backend;
pub use v3::{P3, V3};

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

#[macro_export]
macro_rules! leak_ptr {
    ($val:expr) => {
        Box::leak(Box::new($val))
    };
}
