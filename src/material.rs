use crate::{color, hit::Interval, noise::Perlin, v3, Color, HitRecord, Ray, Rng, P3};
use image::{open, RgbImage};
use rand::RngExt;

#[derive(Debug, Clone, Copy)]
pub enum Texture {
    SolidColor {
        albedo: Color,
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

    pub fn image(path: &str) -> Texture {
        let raw = Box::leak(Box::new(open(path).unwrap().into_rgb8()));

        Self::Image { raw }
    }

    pub fn noise(scale: f32, rng: &mut Rng) -> Texture {
        Self::Noise {
            noise: Box::leak(Box::new(Perlin::new(rng))),
            scale,
        }
    }

    pub fn value(&self, u: f32, v: f32, p: P3) -> Color {
        match self {
            Self::SolidColor { albedo } => *albedo,
            Self::Image { raw } => image_value(u, v, p, raw),
            Self::Noise { noise, scale } => noise_value(p, noise, *scale),
        }
    }
}

fn image_value(mut u: f32, mut v: f32, _p: P3, raw: &RgbImage) -> Color {
    // Clamp input texture coordinates to [0,1] x [1,0]
    u = Interval::UNIT.clamp(u);
    v = 1.0 - Interval::UNIT.clamp(v); // Flip V to image coordinates

    let i = (u * (raw.width() - 1) as f32) as u32;
    let j = (v * (raw.height() - 1) as f32) as u32;
    let px = raw.get_pixel(i, j);
    let scale = 1.0 / 255.0;

    Color::new(
        scale * px.0[0] as f32,
        scale * px.0[1] as f32,
        scale * px.0[2] as f32,
    )
}

fn noise_value(p: P3, noise: &Perlin<256>, scale: f32) -> Color {
    Color::splat(0.5) * (1.0 + (scale * p.z + 10.0 * noise.turb(p, 7)).sin())
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
        albedo: Color,
    },
    DiffuseLight {
        texture: Texture,
    },
    Isotropic {
        texture: Texture,
    },
}

impl Material {
    pub fn needs_uv_calc(&self) -> bool {
        matches!(
            self,
            Self::DiffuseLight {
                texture: Texture::Image { .. },
            } | Self::Isotropic {
                texture: Texture::Image { .. },
            } | Self::Lambertian {
                texture: Texture::Image { .. },
            }
        )
    }

    pub fn solid_color(albedo: Color) -> Material {
        Self::Lambertian {
            texture: Texture::solid(albedo),
        }
    }

    pub fn image(path: &str) -> Material {
        Self::Lambertian {
            texture: Texture::image(path),
        }
    }

    pub fn noise(scale: f32, rng: &mut Rng) -> Material {
        Self::Lambertian {
            texture: Texture::noise(scale, rng),
        }
    }

    pub fn metal(albedo: Color, fuzz: f32) -> Material {
        let fuzz = if fuzz < 1.0 { fuzz } else { 1.0 };

        Self::Metal { albedo, fuzz }
    }

    pub fn dielectric(ref_index: f32, albedo: Color) -> Material {
        Self::Dielectric { ref_index, albedo }
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

    pub fn scatter(
        &self,
        r_in: &Ray,
        r_out: &mut Ray,
        hr: &HitRecord,
        rng: &mut Rng,
    ) -> Option<Color> {
        match self {
            Self::Lambertian { texture } => lambertian_scatter(texture, r_out, hr, rng),
            Self::Specular {
                albedo,
                spec_albedo,
                smoothness,
                prob,
            } => specular_scatter(
                albedo,
                spec_albedo,
                *smoothness,
                *prob,
                r_in,
                r_out,
                hr,
                rng,
            ),
            Self::Metal { albedo, fuzz } => metal_scatter(albedo, *fuzz, r_in, r_out, hr, rng),
            Self::Dielectric { ref_index, albedo } => {
                dielectric_scatter(*ref_index, albedo, r_in, r_out, hr, rng)
            }
            Self::Isotropic { texture } => isotropic_scatter(texture, r_out, hr, rng),
            Self::DiffuseLight { .. } => None,
        }
    }

    pub fn color_emitted(&self, u: f32, v: f32, p: P3) -> Color {
        match self {
            Self::DiffuseLight { texture } => texture.value(u, v, p),
            _ => color::BLACK,
        }
    }
}

fn lambertian_scatter(
    texture: &Texture,
    r_out: &mut Ray,
    hr: &HitRecord,
    rng: &mut Rng,
) -> Option<Color> {
    let mut scatter_direction = hr.normal + v3::random_unit_vector(rng);
    if v3::near_zero(&scatter_direction) {
        scatter_direction = hr.normal;
    }
    let attenuation = texture.value(hr.u, hr.v, hr.p);
    r_out.set(hr.p, scatter_direction);

    Some(attenuation)
}

fn metal_scatter(
    albedo: &Color,
    fuzz: f32,
    r_in: &Ray,
    r_out: &mut Ray,
    hr: &HitRecord,
    rng: &mut Rng,
) -> Option<Color> {
    let reflected = r_in.dir.reflect(hr.normal).normalize() + (fuzz * v3::random_unit_vector(rng));
    r_out.set(hr.p, reflected);

    if r_out.dir.dot(hr.normal) > 0.0 {
        Some(*albedo)
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
    r_out: &mut Ray,
    hr: &HitRecord,
    rng: &mut Rng,
) -> Option<Color> {
    let diffuse_dir = hr.normal + v3::random_unit_vector(rng);
    let is_specular = prob > rng.random_range(0.0..1.0);
    let (dir, color) = if is_specular {
        let specular_dir = r_in.dir.reflect(hr.normal);
        (diffuse_dir.lerp(specular_dir, smoothness), *spec_albedo)
    } else {
        (diffuse_dir, *albedo)
    };

    r_out.set(hr.p, dir);

    Some(color)
}

fn dielectric_scatter(
    ref_index: f32,
    albedo: &Color,
    r_in: &Ray,
    r_out: &mut Ray,
    hr: &HitRecord,
    rng: &mut Rng,
) -> Option<Color> {
    let ri = if hr.front_face {
        1.0 / ref_index
    } else {
        ref_index
    };
    let unit_dir = r_in.dir.normalize();

    let cos_theta = (-unit_dir.dot(hr.normal)).min(1.0);
    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
    let cannot_refract = ri * sin_theta > 1.0;

    let dir = if cannot_refract || reflectance(cos_theta, ri) > rng.random_range(0.0..1.0) {
        unit_dir.reflect(hr.normal)
    } else {
        unit_dir.refract(hr.normal, ri)
    };

    r_out.set(hr.p, dir);

    Some(*albedo)
}

/// Use Schlick's approximation for reflectance.
fn reflectance(cosine: f32, ref_index: f32) -> f32 {
    let r0 = (1.0 - ref_index) / (1.0 + ref_index);
    let r0_sq = r0 * r0;

    r0_sq + (1.0 - r0_sq) * (1.0 - cosine).powi(5)
}

fn isotropic_scatter(
    texture: &Texture,
    r_out: &mut Ray,
    hr: &HitRecord,
    rng: &mut Rng,
) -> Option<Color> {
    r_out.set(hr.p, v3::random_unit_vector(rng));
    let attenuation = texture.value(hr.u, hr.v, hr.p);

    Some(attenuation)
}
