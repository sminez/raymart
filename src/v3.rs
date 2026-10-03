//! A simple 3D vector using f32s
use crate::Rng;
pub use glam::Vec3 as V3;
use rand::RngExt;
use std::f32::consts::PI;

pub type P3 = V3;

const NEAR_ZERO: f32 = 1e-8;

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

pub fn random_on_sphere(r_sq: f32, d_sq: f32, rng: &mut Rng) -> V3 {
    let (a, b): (f32, f32) = rng.random();
    let z = 1.0 + b * ((1.0 - r_sq / d_sq).max(0.0).sqrt() - 1.0);
    let phi = 2.0 * PI * a;
    let (sin_phi, cos_phi) = phi.sin_cos();
    let k = (1.0 - z * z).max(0.0).sqrt();

    V3::new(cos_phi * k, sin_phi * k, z)
}

pub fn random_cosine_direction(rng: &mut Rng) -> V3 {
    let (r1, r2): (f32, f32) = rng.random();
    let phi = 2.0 * PI * r1;
    let sqrt_r2 = r2.sqrt();
    let (sin_phi, cos_phi) = phi.sin_cos();

    V3::new(cos_phi * sqrt_r2, sin_phi * sqrt_r2, (1.0 - r2).sqrt())
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

/// Orthonormal basis
#[derive(Debug, Clone, Copy)]
pub struct Onb {
    pub u: V3,
    pub v: V3,
    pub w: V3,
}

impl Onb {
    pub fn new(normal: V3) -> Self {
        let w = normal.normalize();
        let (u, v) = w.any_orthonormal_pair();

        Self { u, v, w }
    }

    pub fn transform(&self, v: V3) -> V3 {
        v.x * self.u + v.y * self.v + v.z * self.w
    }
}
