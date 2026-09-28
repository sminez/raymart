use crate::{color, leak_ptr, material::Material, v3, Color, HitRecord, Ray, Rng};
use rand::RngExt;

#[derive(Debug, Clone)]
pub struct Specular {
    pub albedo: Color,
    pub spec_albedo: Color,
    pub smoothness: f32,
    pub prob: f32,
}

impl Specular {
    pub fn new_mat(
        albedo: impl Into<Color>,
        spec_albedo: impl Into<Color>,
        smoothness: f32,
        prob: f32,
    ) -> &'static dyn Material {
        leak_ptr!(Self {
            albedo: albedo.into(),
            spec_albedo: spec_albedo.into(),
            smoothness,
            prob,
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

    fn scattering_pdf(&self, _r_in: &Ray, _r_out: &Ray, _hr: &HitRecord) -> f32 {
        0.0
    }

    fn scatter(
        &self,
        r_in: &Ray,
        r_out: &mut Ray,
        hr: &HitRecord,
        _pdf: &mut f32,
        rng: &mut Rng,
    ) -> Option<Color> {
        let diffuse_dir = hr.normal + v3::random_unit_vector(rng);
        let is_specular = self.prob > rng.random_range(0.0..1.0);
        let (dir, color) = if is_specular {
            let specular_dir = r_in.dir.reflect(hr.normal);
            (
                diffuse_dir.lerp(specular_dir, self.smoothness),
                self.spec_albedo,
            )
        } else {
            (diffuse_dir, self.albedo)
        };

        r_out.set(hr.p, dir);

        Some(color)
    }
}
