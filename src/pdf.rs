use crate::{
    v3::{random_cosine_direction, random_unit_vector, Onb},
    Rng, V3,
};
use std::f32::consts::{FRAC_1_PI, PI};

const INV_4PI: f32 = 1.0 / (4.0 * PI);

pub trait Pdf {
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
