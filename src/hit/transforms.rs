use crate::{
    bvh::AABBox,
    hit::{Hittable, Interval},
    material::{Isotropic, Material},
    texture::{SolidColor, Texture},
    Color, HitRecord, Ray, Rng, P3, V3,
};
use glam::Mat3;
use rand::random_range;

#[derive(Debug, Clone)]
pub struct ConstantMedium {
    boundary: &'static dyn Hittable,
    neg_inv_density: f32,
    phase_func: &'static dyn Material,
}

impl ConstantMedium {
    pub fn new(boundary: &'static dyn Hittable, density: f32, albedo: Color) -> ConstantMedium {
        Self::new_with_texture(
            boundary,
            density,
            Box::leak(Box::new(SolidColor::new(albedo))),
        )
    }

    pub fn new_with_texture(
        boundary: &'static dyn Hittable,
        density: f32,
        texture: &'static dyn Texture,
    ) -> ConstantMedium {
        let neg_inv_density = -1.0 / density;

        Self {
            boundary,
            neg_inv_density,
            phase_func: Isotropic::new_mat(texture),
        }
    }
}

impl Hittable for ConstantMedium {
    fn bounding_box(&self) -> AABBox {
        self.boundary.bounding_box()
    }

    fn hits(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
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

    fn pdf_value(&self, origin: P3, dir: V3) -> f32 {
        self.boundary.pdf_value(origin, dir)
    }

    fn random_dir(&self, origin: V3, rng: &mut Rng) -> V3 {
        self.boundary.random_dir(origin, rng)
    }
}

#[derive(Debug, Clone)]
pub struct Translate {
    inner: &'static dyn Hittable,
    offset: V3,
    bbox: AABBox,
}

impl Translate {
    pub fn new(inner: &'static dyn Hittable, offset: V3) -> Translate {
        let bbox = inner.bounding_box() + offset;

        Self {
            inner,
            offset,
            bbox,
        }
    }
}

impl Hittable for Translate {
    fn bounding_box(&self) -> AABBox {
        self.bbox
    }

    fn hits(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        // Move the ray back by the offset
        let offset_r = Ray::new(r.orig - self.offset, r.dir);

        // If the offset ray hits...
        let mut hr = self.inner.hits(&offset_r, ray_t)?;
        // apply the offset to the hit record and return
        hr.p += self.offset;

        Some(hr)
    }

    fn pdf_value(&self, origin: P3, dir: V3) -> f32 {
        self.inner.pdf_value(origin - self.offset, dir)
    }

    fn random_dir(&self, origin: V3, rng: &mut Rng) -> V3 {
        self.inner.random_dir(origin - self.offset, rng)
    }
}

/// Rotation around y
#[derive(Debug, Clone)]
pub struct Rotate {
    inner: &'static dyn Hittable,
    to_obj: Mat3,
    to_world: Mat3,
    bbox: AABBox,
}

impl Rotate {
    pub fn new(inner: &'static dyn Hittable, angle: f32) -> Rotate {
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

        Self {
            inner,
            to_obj,
            to_world,
            bbox: AABBox::new_from_points(min, max),
        }
    }
}

impl Hittable for Rotate {
    fn bounding_box(&self) -> AABBox {
        self.bbox
    }

    fn hits(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        // Transform the ray from world space to object space.
        let rot_r = Ray::new(self.to_obj * r.orig, self.to_obj * r.dir);

        // If the rotated ray hits...
        let mut hr = self.inner.hits(&rot_r, ray_t)?;

        // apply the rotation to the hit record and return
        hr.p = self.to_world * hr.p;
        hr.normal = self.to_world * hr.normal;

        Some(hr)
    }

    fn pdf_value(&self, origin: P3, dir: V3) -> f32 {
        self.inner
            .pdf_value(self.to_obj * origin, self.to_obj * dir)
    }

    fn random_dir(&self, origin: V3, rng: &mut Rng) -> V3 {
        self.to_world * self.inner.random_dir(self.to_obj * origin, rng)
    }
}
