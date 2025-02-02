use crate::{Color, HitRecord, Ray, V3};
use rand::random_range;

#[derive(Debug, Clone, Copy)]
pub enum Material {
    Lambertian { albedo: Color },
    Metal { albedo: Color, fuzz: f64 },
    Dielectric { ref_index: f64 },
}

impl Material {
    pub fn lambertian(albedo: Color) -> Material {
        Self::Lambertian { albedo }
    }

    pub fn metal(albedo: Color, fuzz: f64) -> Material {
        let fuzz = if fuzz < 1.0 { fuzz } else { 1.0 };

        Self::Metal { albedo, fuzz }
    }

    pub fn dielectric(ref_index: f64) -> Material {
        Self::Dielectric { ref_index }
    }

    pub fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
        match self {
            Self::Lambertian { albedo } => lambertian_scatter(albedo, rec),
            Self::Metal { albedo, fuzz } => metal_scatter(albedo, *fuzz, r_in, rec),
            Self::Dielectric { ref_index } => dielectric_scatter(*ref_index, r_in, rec),
        }
    }
}

fn lambertian_scatter(albedo: &Color, rec: &HitRecord) -> Option<(Ray, Color)> {
    let mut scatter_direction = rec.normal + V3::random_unit_vector();
    if scatter_direction.near_zero() {
        scatter_direction = rec.normal;
    }
    let scattered = Ray::new(rec.p, scatter_direction);

    Some((scattered, *albedo))
}

fn metal_scatter(albedo: &Color, fuzz: f64, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
    let reflected = r_in.dir.reflect(rec.normal).unit_vector() + (fuzz * V3::random_unit_vector());
    let scattered = Ray::new(rec.p, reflected);

    if scattered.dir.dot(&rec.normal) > 0.0 {
        Some((scattered, *albedo))
    } else {
        None
    }
}

fn dielectric_scatter(ref_index: f64, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
    let ri = if rec.front_face {
        1.0 / ref_index
    } else {
        ref_index
    };
    let unit_dir = r_in.dir.unit_vector();

    let cos_theta = (-unit_dir.dot(&rec.normal)).min(1.0);
    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
    let cannot_refract = ri * sin_theta > 1.0;

    let direction = if cannot_refract || reflectance(cos_theta, ri) > random_range(0.0..1.0) {
        unit_dir.reflect(rec.normal)
    } else {
        unit_dir.refract(rec.normal, ri)
    };

    Some((Ray::new(rec.p, direction), Color::new(1.0, 1.0, 1.0)))
}

/// Use Schlick's approximation for reflectance.
fn reflectance(cosine: f64, ref_index: f64) -> f64 {
    let r0 = (1.0 - ref_index) / (1.0 + ref_index);
    let r0_sq = r0 * r0;

    r0_sq + (1.0 - r0_sq) * (1.0 - cosine).powi(5)
}
