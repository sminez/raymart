//! A simple 3D vector using f32s
use crate::Rng;
pub use glam::Vec3 as V3;
use rand::RngExt;

pub type P3 = V3;

const NEAR_ZERO: f32 = 1e-8;
pub const ORIGIN: V3 = V3::new(0.0, 0.0, 0.0);

pub const fn new(x: f32, y: f32, z: f32) -> V3 {
    V3 { x, y, z }
}

pub fn random(min: f32, max: f32, rng: &mut Rng) -> V3 {
    V3::new(
        rng.random_range(min..max),
        rng.random_range(min..max),
        rng.random_range(min..max),
    )
}

pub fn random_unit_vector(rng: &mut Rng) -> V3 {
    loop {
        let p = random(-1.0, 1.0, rng);
        let sq_len = p.length_squared();
        if 1e-160 < sq_len && sq_len < 1.0 {
            return p / sq_len.sqrt(); // avoiding computing sq_len again
        }
    }
}

pub fn random_on_hemisphere(normal: V3, rng: &mut Rng) -> V3 {
    let v = random_unit_vector(rng);
    if v.dot(normal) > 0.0 {
        v // Same hemisphere as `normal`
    } else {
        -v
    }
}

pub fn random_in_unit_disk(rng: &mut Rng) -> V3 {
    loop {
        let p = V3::new(
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
            0.0,
        );
        if p.length_squared() < 1.0 {
            return p;
        }
    }
}

pub fn near_zero(v: &V3) -> bool {
    v.abs().max_element() < NEAR_ZERO
}
