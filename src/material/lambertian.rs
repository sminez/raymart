use std::f32::consts::PI;

use crate::{
    color, leak_ptr,
    material::Material,
    texture::{Checker, Image, PerlinNoise, SolidColor, Texture},
    v3::{self, random_cosine_direction, Onb},
    Color, HitRecord, Ray, Rng,
};

#[derive(Debug, Clone)]
pub struct Lambertian {
    pub texture: &'static dyn Texture,
}

impl Lambertian {
    pub fn solid_color(albedo: impl Into<Color>) -> &'static dyn Material {
        leak_ptr!(Self {
            texture: leak_ptr!(SolidColor::new(albedo)),
        })
    }

    pub fn checker(
        scale: f32,
        odd: impl Into<Color>,
        even: impl Into<Color>,
    ) -> &'static dyn Material {
        leak_ptr!(Self {
            texture: leak_ptr!(Checker::new(
                scale,
                leak_ptr!(SolidColor::new(odd)),
                leak_ptr!(SolidColor::new(even)),
            )),
        })
    }

    pub fn image(path: &str) -> &'static dyn Material {
        leak_ptr!(Self {
            texture: leak_ptr!(Image::new(path)),
        })
    }

    pub fn noise(scale: f32, albedo: impl Into<Color>, rng: &mut Rng) -> &'static dyn Material {
        leak_ptr!(Self {
            texture: leak_ptr!(PerlinNoise::new(scale, albedo.into(), rng)),
        })
    }
}

impl Material for Lambertian {
    fn needs_uv_calc(&self) -> bool {
        self.texture.needs_uv_calc()
    }

    fn color_emitted(&self, _hr: &HitRecord) -> Color {
        color::BLACK
    }

    fn scattering_pdf(&self, _r_in: &Ray, r_out: &Ray, hr: &HitRecord) -> f32 {
        let cos_theta = hr.normal.dot(r_out.dir); // r_out.dir already normalized

        if cos_theta < 0.0 {
            0.0
        } else {
            cos_theta / PI
        }
    }

    fn scatter(
        &self,
        _r_in: &Ray,
        r_out: &mut Ray,
        hr: &HitRecord,
        pdf: &mut f32,
        rng: &mut Rng,
    ) -> Option<Color> {
        let onb = Onb::new(hr.normal);
        let mut scatter_direction = onb.transform(random_cosine_direction(rng));
        if v3::near_zero(&scatter_direction) {
            scatter_direction = hr.normal;
        }
        let attenuation = self.texture.value(hr.u, hr.v, hr.p);
        let dir = scatter_direction.normalize();

        r_out.set(hr.p, dir);
        *pdf = onb.w.dot(dir) / PI;

        Some(attenuation)
    }
}
