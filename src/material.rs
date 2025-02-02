use crate::{Color, HitRecord, Ray, V3};

#[derive(Debug, Clone, Copy)]
pub enum Material {
    Lambertian { albedo: Color },
    Metal { albedo: Color },
}

impl Material {
    pub fn lambertian(albedo: Color) -> Material {
        Self::Lambertian { albedo }
    }

    pub fn metal(albedo: Color) -> Material {
        Self::Metal { albedo }
    }

    pub fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
        match self {
            Self::Lambertian { albedo } => lambertian_scatter(albedo, rec),
            Self::Metal { albedo } => metal_scatter(albedo, r_in, rec),
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

fn metal_scatter(albedo: &Color, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
    let reflected = r_in.dir.reflect(rec.normal);
    let scattered = Ray::new(rec.p, reflected);

    Some((scattered, *albedo))
}
