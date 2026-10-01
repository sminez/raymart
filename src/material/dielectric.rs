use crate::{
    color, leak_ptr,
    material::{Material, ScatterRecord},
    Color, HitRecord, Ray, Rng,
};
use rand::RngExt;

#[derive(Debug, Clone, Copy)]
pub struct Dielectric {
    pub albedo: Color,
    ref_index: f32,
    inv_ref_index: f32,
    r0_sq: f32,
    inv_r0_sq: f32,
}

impl Dielectric {
    pub fn new_mat(albedo: impl Into<Color>, ref_index: f32) -> &'static dyn Material {
        // Pre-computed intermediate values for reflectance
        let r0 = (1.0 - ref_index) / (1.0 + ref_index);
        let r0_sq = r0 * r0;

        let inv_ref_index = 1.0 / ref_index;
        let inv_r0 = (1.0 - inv_ref_index) / (1.0 + inv_ref_index);
        let inv_r0_sq = inv_r0 * inv_r0;

        leak_ptr!(Self {
            albedo: albedo.into(),
            ref_index,
            inv_ref_index,
            r0_sq,
            inv_r0_sq
        })
    }
}

impl Material for Dielectric {
    fn needs_uv_calc(&self) -> bool {
        false
    }

    fn color_emitted(&self, _hr: &HitRecord) -> Color {
        color::BLACK
    }

    fn scattering_pdf(&self, _r_in: &Ray, _r_out: &Ray, _hr: &HitRecord) -> f32 {
        0.0
    }

    fn scatter(&self, r_in: &Ray, hr: &HitRecord, rng: &mut Rng) -> Option<ScatterRecord> {
        let (ri, r0_rq) = if hr.front_face {
            (self.inv_ref_index, self.inv_r0_sq)
        } else {
            (self.ref_index, self.r0_sq)
        };
        let unit_dir = r_in.dir.normalize();

        let cos_theta = (-unit_dir.dot(hr.normal)).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let cannot_refract = ri * sin_theta > 1.0;

        let dir = if cannot_refract || reflectance(cos_theta, r0_rq) > rng.random_range(0.0..1.0) {
            unit_dir.reflect(hr.normal)
        } else {
            unit_dir.refract(hr.normal, ri)
        };

        Some(ScatterRecord::Skip {
            attenuation: self.albedo,
            ray: Ray::new(hr.p, dir),
        })
    }
}

/// Use Schlick's approximation for reflectance.
fn reflectance(cosine: f32, r0_sq: f32) -> f32 {
    r0_sq + (1.0 - r0_sq) * (1.0 - cosine).powi(5)
}
