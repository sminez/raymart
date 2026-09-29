use crate::{
    color, leak_ptr,
    material::{Material, ScatterRecord},
    pdf::{CosinePdf, GlossyPdf, WeightedMixturePdf},
    Color, HitRecord, Ray, Rng,
};
use std::f32::consts::FRAC_1_PI;

#[derive(Debug, Clone)]
pub struct Specular {
    pub albedo: Color,
    pub exponent: f32,
    pub prob: f32,
}

impl Specular {
    pub fn new_mat(albedo: impl Into<Color>, smoothness: f32, prob: f32) -> &'static dyn Material {
        let roughness = (1.0 - smoothness).clamp(0.001, 1.0);
        let exponent = (2.0 / (roughness * roughness) - 2.0).max(0.0);

        leak_ptr!(Self {
            albedo: albedo.into(),
            exponent,
            prob: prob.clamp(0.0, 1.0),
        })
    }
}

impl Material for Specular {
    fn needs_uv_calc(&self) -> bool {
        false
    }

    fn color_emitted(&self, _hr: &HitRecord) -> Color {
        color::BLACK
    }

    fn scattering_pdf(&self, _r_in: &Ray, r_out: &Ray, hr: &HitRecord) -> f32 {
        let cos_theta = hr.normal.dot(r_out.dir); // r_out.dir already normalized

        if cos_theta < 0.0 {
            0.0
        } else {
            cos_theta * FRAC_1_PI
        }
    }

    fn scatter(&self, r_in: &Ray, hr: &HitRecord, _rng: &mut Rng) -> Option<ScatterRecord> {
        // let reflect_dir = r_in.dir.normalize().reflect(hr.normal).normalize();
        let reflect_dir = r_in.dir.reflect(hr.normal);

        Some(ScatterRecord::Pdf {
            attenuation: self.albedo,
            pdf: Box::new(WeightedMixturePdf::new(
                self.prob,
                GlossyPdf::new(reflect_dir, hr.normal, self.exponent),
                CosinePdf::new(hr.normal),
            )),
        })
    }
}
