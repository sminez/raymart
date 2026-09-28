use crate::{Color, HitRecord, Ray, Rng};
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
    fn scatter(&self, r_in: &Ray, r_out: &mut Ray, hr: &HitRecord, rng: &mut Rng) -> Option<Color>;
    fn scattering_pdf(&self, r_in: &Ray, r_out: &Ray, hr: &HitRecord) -> f32;
}
