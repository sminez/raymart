use crate::{hit::Interval, noise::Perlin, Color, Rng, P3};
use image::{open, RgbImage};
use std::{fmt, sync::Arc};

pub trait Texture: fmt::Debug + Send + Sync {
    // cant be an associated constant as we need the trait to be dyn-compatible
    fn needs_uv_calc(&self) -> bool;

    fn value(&self, u: f32, v: f32, p: P3) -> Color;
}

#[derive(Debug, Clone, Copy)]
pub struct SolidColor {
    pub albedo: Color,
}

impl SolidColor {
    pub fn new(albedo: impl Into<Color>) -> Self {
        Self {
            albedo: albedo.into(),
        }
    }
}

impl Texture for SolidColor {
    fn needs_uv_calc(&self) -> bool {
        false
    }

    fn value(&self, _u: f32, _v: f32, _p: P3) -> Color {
        self.albedo
    }
}

#[derive(Debug, Clone)]
pub struct Checker {
    pub inv_scale: f32,
    pub odd: &'static dyn Texture,
    pub even: &'static dyn Texture,
}

impl Checker {
    pub fn new(scale: f32, odd: &'static dyn Texture, even: &'static dyn Texture) -> Self {
        Self {
            inv_scale: 1.0 / scale,
            odd,
            even,
        }
    }
}

impl Texture for Checker {
    fn needs_uv_calc(&self) -> bool {
        true
    }

    fn value(&self, u: f32, v: f32, p: P3) -> Color {
        let x = (self.inv_scale * p.x).floor() as i64;
        let y = (self.inv_scale * p.y).floor() as i64;
        let z = (self.inv_scale * p.z).floor() as i64;

        if (x + y + z) % 2 == 0 {
            self.even.value(u, v, p)
        } else {
            self.odd.value(u, v, p)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Image {
    pub raw: Arc<RgbImage>,
}

impl Image {
    pub fn new(path: &str) -> Self {
        let raw = Arc::new(open(path).unwrap().into_rgb8());

        Self { raw }
    }
}

impl Texture for Image {
    fn needs_uv_calc(&self) -> bool {
        true
    }

    fn value(&self, mut u: f32, mut v: f32, _p: P3) -> Color {
        // Clamp input texture coordinates to [0,1] x [1,0]
        u = Interval::UNIT.clamp(u);
        v = 1.0 - Interval::UNIT.clamp(v); // Flip V to image coordinates

        let i = (u * (self.raw.width() - 1) as f32) as u32;
        let j = (v * (self.raw.height() - 1) as f32) as u32;
        let px = self.raw.get_pixel(i, j);
        let scale = 1.0 / 255.0;

        Color::new(
            scale * px.0[0] as f32,
            scale * px.0[1] as f32,
            scale * px.0[2] as f32,
        )
    }
}

#[derive(Debug, Clone)]
pub struct PerlinNoise {
    pub noise: Arc<Perlin<256>>,
    pub scale: f32,
    pub albedo: Color,
}

impl PerlinNoise {
    pub fn new(scale: f32, albedo: Color, rng: &mut Rng) -> Self {
        Self {
            noise: Arc::new(Perlin::new(rng)),
            scale,
            albedo,
        }
    }
}

impl Texture for PerlinNoise {
    fn needs_uv_calc(&self) -> bool {
        false
    }

    fn value(&self, _u: f32, _v: f32, p: P3) -> Color {
        self.noise.noise_value(p, self.scale, self.albedo)
    }
}
