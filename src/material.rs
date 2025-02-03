use crate::{Color, HitRecord, Ray, P3, V3};
use rand::random_range;

#[derive(Debug, Clone, Copy)]
pub enum Texture {
    SolidColor {
        albedo: Color,
    },
    Checker {
        inv_scale: f64,
        odd: &'static Texture,
        even: &'static Texture,
    },
}

impl Texture {
    pub fn solid(albedo: Color) -> Texture {
        Self::SolidColor { albedo }
    }

    pub fn checker(scale: f64, odd: Texture, even: Texture) -> Texture {
        Self::Checker {
            inv_scale: 1.0 / scale,
            odd: Box::leak(Box::new(odd)),
            even: Box::leak(Box::new(even)),
        }
    }

    pub fn value(&self, u: f64, v: f64, p: P3) -> Color {
        match self {
            Self::SolidColor { albedo } => *albedo,
            Self::Checker {
                inv_scale,
                odd,
                even,
            } => checker_value(u, v, p, *inv_scale, odd, even),
        }
    }
}

fn checker_value(u: f64, v: f64, p: P3, inv_scale: f64, odd: &Texture, even: &Texture) -> Color {
    let x = (inv_scale * p.x).floor() as i64;
    let y = (inv_scale * p.y).floor() as i64;
    let z = (inv_scale * p.z).floor() as i64;

    if (x + y + z) % 2 == 0 {
        even.value(u, v, p)
    } else {
        odd.value(u, v, p)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Material {
    Lambertian { texture: Texture },
    Metal { albedo: Color, fuzz: f64 },
    Dielectric { ref_index: f64 },
}

impl Material {
    pub fn solid_color(albedo: Color) -> Material {
        Self::Lambertian {
            texture: Texture::solid(albedo),
        }
    }

    pub fn checker(scale: f64, even: Color, odd: Color) -> Material {
        Self::Lambertian {
            texture: Texture::checker(scale, Texture::solid(even), Texture::solid(odd)),
        }
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
            Self::Lambertian { texture } => lambertian_scatter(texture, r_in, rec),
            Self::Metal { albedo, fuzz } => metal_scatter(albedo, *fuzz, r_in, rec),
            Self::Dielectric { ref_index } => dielectric_scatter(*ref_index, r_in, rec),
        }
    }
}

fn lambertian_scatter(texture: &Texture, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
    let mut scatter_direction = rec.normal + V3::random_unit_vector();
    if scatter_direction.near_zero() {
        scatter_direction = rec.normal;
    }
    let scattered = Ray::new(rec.p, scatter_direction, r_in.time);
    let attenuation = texture.value(rec.u, rec.v, rec.p);

    Some((scattered, attenuation))
}

fn metal_scatter(albedo: &Color, fuzz: f64, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
    let reflected = r_in.dir.reflect(rec.normal).unit_vector() + (fuzz * V3::random_unit_vector());
    let scattered = Ray::new(rec.p, reflected, r_in.time);

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

    Some((
        Ray::new(rec.p, direction, r_in.time),
        Color::new(1.0, 1.0, 1.0),
    ))
}

/// Use Schlick's approximation for reflectance.
fn reflectance(cosine: f64, ref_index: f64) -> f64 {
    let r0 = (1.0 - ref_index) / (1.0 + ref_index);
    let r0_sq = r0 * r0;

    r0_sq + (1.0 - r0_sq) * (1.0 - cosine).powi(5)
}
