use crate::{
    bvh::AABBox,
    hit::{Hittable, Interval},
    material::{Material, Texture},
    Color, HitRecord, Ray, P3, V3,
};
use glam::Mat3;
use rand::random_range;

#[derive(Debug, Clone)]
pub struct ConstantMedium {
    boundary: &'static Hittable,
    neg_inv_density: f32,
    phase_func: &'static Material,
}

impl ConstantMedium {
    pub fn new(boundary: Hittable, density: f32, color: Color) -> ConstantMedium {
        Self::new_with_texture(boundary, density, Texture::solid(color))
    }

    pub fn new_with_texture(boundary: Hittable, density: f32, texture: Texture) -> ConstantMedium {
        let neg_inv_density = -1.0 / density;

        Self {
            boundary: Box::leak(Box::new(boundary)),
            neg_inv_density,
            phase_func: Box::leak(Box::new(Material::isotropic_texture(texture))),
        }
    }

    pub fn bounding_box(&self) -> AABBox {
        self.boundary.bounding_box()
    }

    pub fn hits(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        let mut hr1 = self.boundary.hits(r, Interval::UNIVERSE)?;
        let i2 = Interval::new(hr1.t + 0.0001, f32::INFINITY);
        let mut hr2 = self.boundary.hits(r, i2)?;

        hr1.t = hr1.t.max(ray_t.min);
        hr2.t = hr2.t.min(ray_t.max);
        if hr1.t > hr2.t {
            return None;
        }

        hr1.t = hr1.t.max(0.0);

        let r_len = r.dir.length();
        let dist_in_boundary = (hr2.t - hr1.t) * r_len;
        let hit_dist = self.neg_inv_density * random_range(0.0..1.0f32).log2();
        if hit_dist > dist_in_boundary {
            return None;
        }

        let t = hr1.t + hit_dist / r_len;
        let normal = V3::new(1.0, 0.0, 0.0); // arbitrary
        let (u, v) = (0.0, 0.0); // arbitrary

        Some(HitRecord::new(t, r.at(t), normal, r, self.phase_func, u, v))
    }
}

#[derive(Debug, Clone)]
pub struct Translate {
    inner: Box<Hittable>,
    offset: V3,
    pub bbox: AABBox,
}

impl Translate {
    pub fn new(inner: Hittable, offset: V3) -> Translate {
        let bbox = inner.bounding_box() + offset;

        Self {
            inner: Box::new(inner),
            offset,
            bbox,
        }
    }

    pub fn hits(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        // Move the ray back by the offset
        let offset_r = Ray::new(r.orig - self.offset, r.dir);

        // If the offset ray hits...
        let mut hr = self.inner.hits(&offset_r, ray_t)?;
        // apply the offset to the hit record and return
        hr.p += self.offset;

        Some(hr)
    }
}

/// Rotation around y
#[derive(Debug, Clone)]
pub struct Rotate {
    inner: Box<Hittable>,
    to_obj: Mat3,
    to_world: Mat3,
    pub bbox: AABBox,
}

impl Rotate {
    pub fn new(inner: Hittable, angle: f32) -> Rotate {
        let to_world = Mat3::from_rotation_y(angle.to_radians());
        let to_obj = to_world.transpose();
        let bbox = inner.bounding_box();

        let mut min = P3::splat(f32::INFINITY);
        let mut max = P3::splat(-f32::INFINITY);

        for x in [bbox.x.min, bbox.x.max] {
            for y in [bbox.y.min, bbox.y.max] {
                for z in [bbox.z.min, bbox.z.max] {
                    let v = to_world * V3::new(x, y, z);
                    min = min.min(v);
                    max = max.max(v);
                }
            }
        }

        let bbox = AABBox::new_from_points(min, max);

        Self {
            inner: Box::new(inner),
            to_obj,
            to_world,
            bbox,
        }
    }

    pub fn hits(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        // Transform the ray from world space to object space.
        let rot_r = Ray::new(self.to_obj * r.orig, self.to_obj * r.dir);

        // If the rotated ray hits...
        let mut hr = self.inner.hits(&rot_r, ray_t)?;

        // apply the rotation to the hit record and return
        hr.p = self.to_world * hr.p;
        hr.normal = self.to_world * hr.normal;

        Some(hr)
    }
}
