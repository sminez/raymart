use crate::{
    color, leak_ptr,
    material::{Material, ScatterRecord},
    pdf::CosinePdf,
    v3, Color, HitRecord, Ray, Rng,
};
use rand::RngExt;
use std::f32::consts::FRAC_1_PI;

#[derive(Debug, Clone)]
pub struct Specular {
    pub albedo: Color,
    pub prob: f32,
    pub smoothness: f32,
}

impl Specular {
    pub fn new_mat(albedo: impl Into<Color>, smoothness: f32, prob: f32) -> &'static dyn Material {
        leak_ptr!(Self {
            albedo: albedo.into(),
            prob: prob.clamp(0.0, 1.0),
            smoothness,
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

    fn scatter(&self, r_in: &Ray, hr: &HitRecord, rng: &mut Rng) -> Option<ScatterRecord> {
        let is_specular = self.prob > rng.random_range(0.0..1.0);
        if is_specular {
            let diffuse_dir = hr.normal + v3::random_unit_vector(rng);
            let specular_dir = r_in.dir.reflect(hr.normal);
            let dir = diffuse_dir.lerp(specular_dir, self.smoothness);

            Some(ScatterRecord::Skip {
                attenuation: self.albedo,
                ray: Ray::new(hr.p, dir),
            })
        } else {
            Some(ScatterRecord::Pdf {
                attenuation: self.albedo,
                pdf: Box::new(CosinePdf::new(hr.normal)),
            })
        }
    }
}
