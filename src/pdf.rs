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
        let cos_theta = dir.normalize().dot(self.basis.w);

        (cos_theta * FRAC_1_PI).max(0.0)
    }

    fn generate(&self, rng: &mut Rng) -> V3 {
        self.basis.transform(random_cosine_direction(rng))
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
        self.objects.random_dir(self.origin, rng).normalize()
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
