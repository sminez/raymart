use crate::{pdf::Pdf, Color, HitRecord, Ray, Rng};
use std::fmt;

mod dielectric;
mod diffuse_light;
mod isotropic;
mod lambertian;
mod metal;
mod specular;

pub use dielectric::Dielectric;
pub use diffuse_light::DiffuseLight;
pub use isotropic::Isotropic;
pub use lambertian::Lambertian;
pub use metal::Metal;
pub use specular::Specular;

pub trait Material: fmt::Debug + Send + Sync {
    fn needs_uv_calc(&self) -> bool;
    fn color_emitted(&self, hr: &HitRecord) -> Color;
    fn scatter(&self, r_in: &Ray, hr: &HitRecord, rng: &mut Rng) -> Option<ScatterRecord>;
    fn scattering_pdf(&self, r_in: &Ray, scattered: &Ray, hr: &HitRecord) -> f32;
}

#[derive(Debug)]
pub enum ScatterRecord {
    Pdf {
        attenuation: Color,
        pdf: Box<dyn Pdf>,
    },
    Skip {
        attenuation: Color,
        ray: Ray,
    },
}
