use crate::{
    color, leak_ptr,
    material::{Material, ScatterRecord},
    v3, Color, HitRecord, Ray, Rng,
};

#[derive(Debug, Clone, Copy)]
pub struct Metal {
    albedo: Color,
    fuzz: f32,
}

impl Metal {
    pub fn new_mat(albedo: impl Into<Color>, fuzz: f32) -> &'static dyn Material {
        let fuzz = if fuzz < 1.0 { fuzz } else { 1.0 };

        leak_ptr!(Self {
            albedo: albedo.into(),
            fuzz,
        })
    }
}

impl Material for Metal {
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
        let reflected =
            r_in.dir.reflect(hr.normal).normalize() + (self.fuzz * v3::random_unit_vector(rng));

        // r_out.set(hr.p, reflected);
        // if r_out.dir.dot(hr.normal) > 0.0 {
        //     Some(self.albedo)
        // } else {
        //     None
        // }

        Some(ScatterRecord::Skip {
            attenuation: self.albedo,
            ray: Ray::new(hr.p, reflected),
        })
    }
}
