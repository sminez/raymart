use crate::{Color, HitRecord, Ray, V3};

#[derive(Debug, Clone, Copy)]
pub enum Material {
    Lambertian { albedo: Color },
    Metal { albedo: Color, fuzz: f64 },
}

impl Material {
    pub fn lambertian(albedo: Color) -> Material {
        Self::Lambertian { albedo }
    }

    pub fn metal(albedo: Color, fuzz: f64) -> Material {
        let fuzz = if fuzz < 1.0 { fuzz } else { 1.0 };

        Self::Metal { albedo, fuzz }
    }

    pub fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
        match self {
            Self::Lambertian { albedo } => lambertian_scatter(albedo, rec),
            Self::Metal { albedo, fuzz } => metal_scatter(albedo, *fuzz, r_in, rec),
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
