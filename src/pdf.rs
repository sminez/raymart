use crate::{
    hit::Hittable,
    v3::{random_cosine_direction, random_unit_vector, Onb},
    Rng, P3, V3,
};
use rand::RngExt;
use std::{
    f32::consts::{FRAC_1_PI, PI},
    fmt,
};

const INV_2PI: f32 = 1.0 / (2.0 * PI);
const INV_4PI: f32 = 1.0 / (4.0 * PI);

pub trait Pdf: fmt::Debug + Send + Sync + 'static {
    fn value(&self, dir: V3) -> f32;
    fn generate(&self, rng: &mut Rng) -> V3;
}

/// PDF returning uniform density over the unit sphere
#[derive(Debug, Clone, Copy)]
pub struct SpherePdf;

impl Pdf for SpherePdf {
    fn value(&self, _dir: V3) -> f32 {
        INV_4PI
    }

    fn generate(&self, rng: &mut Rng) -> V3 {
        random_unit_vector(rng)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CosinePdf {
    basis: Onb,
}

impl CosinePdf {
    pub fn new(w: V3) -> Self {
        Self { basis: Onb::new(w) }
    }
}

impl Pdf for CosinePdf {
    fn value(&self, dir: V3) -> f32 {
        let len_sq = dir.length_squared();
        if !len_sq.is_finite() || len_sq <= f32::EPSILON {
            return 0.0;
        }

        let cos_theta = dir.normalize().dot(self.basis.w);

        (cos_theta * FRAC_1_PI).max(0.0)
    }

    fn generate(&self, rng: &mut Rng) -> V3 {
        self.basis.transform(random_cosine_direction(rng))
    }
}

// Phong style BRDF
#[derive(Debug, Clone, Copy)]
pub struct GlossyPdf {
    basis: Onb,
    normal: V3,
    exponent: f32,
    inv_exponent_plus_1: f32,
}

impl GlossyPdf {
    pub fn new(reflect_dir: V3, normal: V3, smoothness: f32) -> Self {
        let roughness = (1.0 - smoothness).clamp(0.001, 1.0);
        let exponent = (2.0 / (roughness * roughness) - 2.0).max(0.0);
        let inv_exponent_plus_1 = 1.0 / (exponent + 1.0);

        Self {
            basis: Onb::new(reflect_dir.normalize()),
            normal: normal.normalize(),
            exponent,
            inv_exponent_plus_1,
        }
    }

    pub fn copy_with_hit_details(&self, reflect_dir: V3, normal: V3) -> Self {
        Self {
            basis: Onb::new(reflect_dir.normalize()),
            normal: normal.normalize(),
            exponent: self.exponent,
            inv_exponent_plus_1: self.inv_exponent_plus_1,
        }
    }
}

impl Pdf for GlossyPdf {
    fn value(&self, dir: V3) -> f32 {
        let len_sq = dir.length_squared();
        if !len_sq.is_finite() || len_sq <= f32::EPSILON {
            return 0.0;
        }

        let dir = dir.normalize();

        if self.normal.dot(dir) <= 0.0 {
            return 0.0;
        }

        let cos_alpha = self.basis.w.dot(dir).max(0.0);

        ((self.exponent + 1.0) * cos_alpha.powf(self.exponent)) * INV_2PI
    }

    fn generate(&self, rng: &mut Rng) -> V3 {
        loop {
            let (u1, u2): (f32, f32) = rng.random();
            let phi = 2.0 * PI * u1;
            let cos_theta = u2.powf(self.inv_exponent_plus_1);
            let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
            let (sin_phi, cos_phi) = phi.sin_cos();
            let local = V3::new(cos_phi * sin_theta, sin_phi * sin_theta, cos_theta);
            let dir = self.basis.transform(local);

            if self.normal.dot(dir) > 0.0 {
                return dir.normalize();
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct HittablePdf {
    objects: &'static dyn Hittable,
    origin: P3,
}

impl HittablePdf {
    pub fn new(objects: &'static dyn Hittable, origin: P3) -> Self {
        Self { objects, origin }
    }
}

impl Pdf for HittablePdf {
    fn value(&self, dir: V3) -> f32 {
        self.objects.pdf_value(self.origin, dir)
    }

    fn generate(&self, rng: &mut Rng) -> V3 {
        let dir = self.objects.random_dir(self.origin, rng);
        let len_sq = dir.length_squared();

        if !len_sq.is_finite() || len_sq <= f32::EPSILON {
            random_unit_vector(rng)
        } else {
            dir / len_sq.sqrt()
        }
    }
}

#[derive(Debug)]
pub struct MixturePdf {
    pub p1: Box<dyn Pdf>,
    pub p2: Box<dyn Pdf>,
}

impl Pdf for MixturePdf {
    fn value(&self, dir: V3) -> f32 {
        0.5 * self.p1.value(dir) + 0.5 * self.p2.value(dir)
    }

    fn generate(&self, rng: &mut Rng) -> V3 {
        if rng.random_bool(0.5) {
            self.p1.generate(rng)
        } else {
            self.p2.generate(rng)
        }
    }
}

#[derive(Debug)]
pub struct WeightedMixturePdf {
    w: f32,
    p1: Box<dyn Pdf>,
    p2: Box<dyn Pdf>,
}

impl WeightedMixturePdf {
    pub fn new(w: f32, p1: impl Pdf, p2: impl Pdf) -> Self {
        Self {
            w: w.clamp(0.0, 1.0),
            p1: Box::new(p1),
            p2: Box::new(p2),
        }
    }
}

impl Pdf for WeightedMixturePdf {
    fn value(&self, dir: V3) -> f32 {
        self.w * self.p1.value(dir) + (1.0 - self.w) * self.p2.value(dir)
    }

    fn generate(&self, rng: &mut Rng) -> V3 {
        if rng.random_range(0.0..1.0) < self.w {
            self.p1.generate(rng)
        } else {
            self.p2.generate(rng)
        }
    }
}
