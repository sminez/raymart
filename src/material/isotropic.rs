use crate::{
    color, leak_ptr,
    material::{Material, ScatterRecord},
    pdf::SpherePdf,
    texture::{SolidColor, Texture},
    Color, HitRecord, Ray, Rng,
};
use std::f32::consts::PI;

const INV_4PI: f32 = 1.0 / (4.0 * PI);

#[derive(Debug, Clone)]
pub struct Isotropic {
    texture: &'static dyn Texture,
}

impl Isotropic {
    pub fn new_mat(texture: &'static dyn Texture) -> &'static dyn Material {
        leak_ptr!(Self { texture })
    }

    pub fn new_color(albedo: impl Into<Color>) -> &'static dyn Material {
        Self::new_mat(leak_ptr!(SolidColor::new(albedo)))
    }
}

impl Material for Isotropic {
    fn needs_uv_calc(&self) -> bool {
        self.texture.needs_uv_calc()
    }

    fn color_emitted(&self, _hr: &HitRecord) -> Color {
        color::BLACK
    }

    fn scattering_pdf(&self, _r_in: &Ray, _r_out: &Ray, _hr: &HitRecord) -> f32 {
        INV_4PI
    }

    fn scatter(&self, _r_in: &Ray, hr: &HitRecord, _rng: &mut Rng) -> Option<ScatterRecord> {
        Some(ScatterRecord::Pdf {
            attenuation: self.texture.value(hr.u, hr.v, hr.p),
            pdf: Box::new(SpherePdf),
        })
    }
}
