use crate::{bvh::AABBox, hit::Interval, material::Material, HitRecord, Ray, P3, V3};
use std::f32::consts::PI;

const INV_PI: f32 = 1.0 / PI;
const INV_2PI: f32 = 1.0 / (2.0 * PI);

#[derive(Debug, Clone)]
pub struct Sphere {
    center: P3,
    inv_radius: f32,
    radius_sq: f32,
    mat: &'static Material,
    pub bbox: AABBox,
}

impl Sphere {
    pub fn new(center: P3, radius: f32, mat: &'static Material) -> Self {
        let r = radius.max(0.0);
        let rvec = V3::splat(r);
        let bbox = AABBox::new_from_points(center - rvec, center + rvec);

        Self {
            center,
            inv_radius: 1.0 / r,
            radius_sq: r * r,
            mat,
            bbox,
        }
    }

    /// The derivation of the calculation here is given in section 5 of Ray tracing in one weekend
    /// https://raytracing.github.io/books/RayTracingInOneWeekend.html
    pub fn hits(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        let oc = self.center - r.orig;

        let a = r.dir.length_squared();
        let h = r.dir.dot(oc);
        let c = oc.length_squared() - self.radius_sq;
        let discriminant = h * h - a * c;

        if discriminant < 0.0 {
            return None;
        }

        let sqrt_disc = discriminant.sqrt();

        // Find the nearest root that lies between tmin & tmax
        let inv_a = 1.0 / a;
        let mut root = (h - sqrt_disc) * inv_a;
        if !ray_t.surrounds(root) {
            root = (h + sqrt_disc) * inv_a;
            if !ray_t.surrounds(root) {
                return None;
            }
        }

        let p = r.at(root);
        let outward_normal = (p - self.center) * self.inv_radius;

        let (u, v) = if self.mat.needs_uv_calc() {
            let theta = (-outward_normal.y).acos();
            let phi = (-outward_normal.z).atan2(outward_normal.x) + PI;
            (phi * INV_2PI, theta * INV_PI)
        } else {
            (0.0, 0.0)
        };

        Some(HitRecord::new(root, p, outward_normal, r, self.mat, u, v))
    }
}
