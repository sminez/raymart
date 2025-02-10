use crate::{hit::Interval, noise::Perlin, Color, HitRecord, Ray, P3, V3};
use image::{open, RgbImage};
use rand::random_range;

#[derive(Debug, Clone, Copy)]
pub enum Texture {
    SolidColor {
        albedo: Color,
    },
    Checker {
        inv_scale: f32,
        odd: &'static Texture,
        even: &'static Texture,
    },
    Image {
        raw: &'static RgbImage,
    },
    Noise {
        noise: &'static Perlin<256>,
        scale: f32,
    },
}

impl Texture {
    pub fn solid(albedo: Color) -> Texture {
        Self::SolidColor { albedo }
    }

    pub fn checker(scale: f32, odd: Texture, even: Texture) -> Texture {
        Self::Checker {
            inv_scale: 1.0 / scale,
            odd: Box::leak(Box::new(odd)),
            even: Box::leak(Box::new(even)),
        }
    }

    pub fn image(path: &str) -> Texture {
        let raw = Box::leak(Box::new(open(path).unwrap().into_rgb8()));

        Self::Image { raw }
    }

    pub fn noise(scale: f32) -> Texture {
        Self::Noise {
            noise: Box::leak(Box::new(Perlin::new())),
            scale,
        }
    }

    pub fn value(&self, u: f32, v: f32, p: P3) -> Color {
        match self {
            Self::SolidColor { albedo } => *albedo,
            Self::Checker {
                inv_scale,
                odd,
                even,
            } => checker_value(u, v, p, *inv_scale, odd, even),
            Self::Image { raw } => image_value(u, v, p, raw),
            Self::Noise { noise, scale } => noise_value(p, noise, *scale),
        }
    }
}

fn checker_value(u: f32, v: f32, p: P3, inv_scale: f32, odd: &Texture, even: &Texture) -> Color {
    let x = (inv_scale * p.x).floor() as i64;
    let y = (inv_scale * p.y).floor() as i64;
    let z = (inv_scale * p.z).floor() as i64;

    if (x + y + z) % 2 == 0 {
        even.value(u, v, p)
    } else {
        odd.value(u, v, p)
    }
}

fn image_value(mut u: f32, mut v: f32, _p: P3, raw: &RgbImage) -> Color {
    // Clamp input texture coordinates to [0,1] x [1,0]
    u = Interval::UNIT.clamp(u);
    v = 1.0 - Interval::UNIT.clamp(v); // Flip V to image coordinates

    let i = (u * raw.width() as f32) as u32;
    let j = (v * raw.height() as f32) as u32;
    let px = raw.get_pixel(i, j);
    let scale = 1.0 / 255.0;

    Color::new(
        scale * px.0[0] as f32,
        scale * px.0[1] as f32,
        scale * px.0[2] as f32,
    )
}

fn noise_value(p: P3, noise: &Perlin<256>, scale: f32) -> Color {
    Color::new(0.5, 0.5, 0.5) * (1.0 + (scale * p.z + 10.0 * noise.turb(p, 7)).sin())
}

#[derive(Debug, Clone, Copy)]
pub enum Material {
    Lambertian {
        texture: Texture,
    },
    Specular {
        albedo: Color,
        spec_albedo: Color,
        smoothness: f32,
        prob: f32,
    },
    Metal {
        albedo: Color,
        fuzz: f32,
    },
    Dielectric {
        ref_index: f32,
    },
    DiffuseLight {
        texture: Texture,
    },
    Isotropic {
        texture: Texture,
    },
}

impl Material {
    pub fn solid_color(albedo: Color) -> Material {
        Self::Lambertian {
            texture: Texture::solid(albedo),
        }
    }

    pub fn checker(scale: f32, even: Color, odd: Color) -> Material {
        Self::Lambertian {
            texture: Texture::checker(scale, Texture::solid(even), Texture::solid(odd)),
        }
    }

    pub fn image(path: &str) -> Material {
        Self::Lambertian {
            texture: Texture::image(path),
        }
    }

    pub fn noise(scale: f32) -> Material {
        Self::Lambertian {
            texture: Texture::noise(scale),
        }
    }

    pub fn metal(albedo: Color, fuzz: f32) -> Material {
        let fuzz = if fuzz < 1.0 { fuzz } else { 1.0 };

        Self::Metal { albedo, fuzz }
    }

    pub fn dielectric(ref_index: f32) -> Material {
        Self::Dielectric { ref_index }
    }

    pub fn diffuse_light(albedo: Color) -> Material {
        Self::DiffuseLight {
            texture: Texture::solid(albedo),
        }
    }

    pub fn diffuse_light_texture(texture: Texture) -> Material {
        Self::DiffuseLight { texture }
    }

    pub fn isotropic(albedo: Color) -> Material {
        Self::Isotropic {
            texture: Texture::solid(albedo),
        }
    }

    pub fn isotropic_texture(texture: Texture) -> Material {
        Self::Isotropic { texture }
    }

    pub fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
        match self {
            Self::Lambertian { texture } => lambertian_scatter(texture, rec),
            Self::Specular {
                albedo,
                spec_albedo,
                smoothness,
                prob,
            } => specular_scatter(albedo, spec_albedo, *smoothness, *prob, r_in, rec),
            Self::Metal { albedo, fuzz } => metal_scatter(albedo, *fuzz, r_in, rec),
            Self::Dielectric { ref_index } => dielectric_scatter(*ref_index, r_in, rec),
            Self::Isotropic { texture } => isotropic_scatter(texture, rec),
            Self::DiffuseLight { .. } => None,
        }
    }

    pub fn color_emitted(&self, u: f32, v: f32, p: P3) -> Color {
        match self {
            Self::DiffuseLight { texture } => texture.value(u, v, p),
            _ => Color::BLACK,
        }
    }
}

fn lambertian_scatter(texture: &Texture, rec: &HitRecord) -> Option<(Ray, Color)> {
    let mut scatter_direction = rec.normal + V3::random_unit_vector();
    if scatter_direction.near_zero() {
        scatter_direction = rec.normal;
    }
    let scattered = Ray::new(rec.p, scatter_direction);
    let attenuation = texture.value(rec.u, rec.v, rec.p);

    Some((scattered, attenuation))
}

fn metal_scatter(albedo: &Color, fuzz: f32, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
    let reflected = r_in.dir.reflect(rec.normal).unit_vector() + (fuzz * V3::random_unit_vector());
    let scattered = Ray::new(rec.p, reflected);

    if scattered.dir.dot(&rec.normal) > 0.0 {
        Some((scattered, *albedo))
    } else {
        None
    }
}

fn specular_scatter(
    albedo: &Color,
    spec_albedo: &Color,
    smoothness: f32,
    prob: f32,
    r_in: &Ray,
    rec: &HitRecord,
) -> Option<(Ray, Color)> {
    let diffuse_dir = rec.normal + V3::random_unit_vector();
    let is_specular = prob > random_range(0.0..1.0);
    let (dir, color) = if is_specular {
        let specular_dir = r_in.dir.reflect(rec.normal);
        (
            diffuse_dir * (1.0 - smoothness) + specular_dir * smoothness,
            *spec_albedo,
        )
    } else {
        (diffuse_dir, *albedo)
    };

    Some((Ray::new(rec.p, dir), color))
}

fn dielectric_scatter(ref_index: f32, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
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

    Some((Ray::new(rec.p, direction), Color::WHITE))
}

/// Use Schlick's approximation for reflectance.
fn reflectance(cosine: f32, ref_index: f32) -> f32 {
    let r0 = (1.0 - ref_index) / (1.0 + ref_index);
    let r0_sq = r0 * r0;

    r0_sq + (1.0 - r0_sq) * (1.0 - cosine).powi(5)
}

fn isotropic_scatter(texture: &Texture, rec: &HitRecord) -> Option<(Ray, Color)> {
    let scattered = Ray::new(rec.p, V3::random_unit_vector());
    let attenuation = texture.value(rec.u, rec.v, rec.p);

    Some((scattered, attenuation))
}
