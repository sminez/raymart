use crate::{
    bvh::AABBox,
    hit::{Hittable, Interval},
    material::Material,
    v3::{random_on_sphere, random_unit_vector, Onb},
    HitRecord, Ray, Rng, P3, V3,
};
use std::f32::consts::PI;

const INV_PI: f32 = 1.0 / PI;
const INV_2PI: f32 = 1.0 / (2.0 * PI);
const INV_4PI: f32 = 1.0 / (4.0 * PI);

#[derive(Debug, Clone)]
pub struct Sphere {
    center: P3,
    inv_radius: f32,
    radius_sq: f32,
    mat: &'static dyn Material,
    pub bbox: AABBox,
}

impl Sphere {
    pub fn new(center: P3, radius: f32, mat: &'static dyn Material) -> Self {
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
}

impl Hittable for Sphere {
    fn bounding_box(&self) -> AABBox {
        self.bbox
    }

    /// The derivation of the calculation here is given in section 5 of Ray tracing in one weekend
    /// https://raytracing.github.io/books/RayTracingInOneWeekend.html
    fn hits(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
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

    fn pdf_value(&self, origin: P3, dir: V3) -> f32 {
        if self
            .hits(&Ray::new(origin, dir), Interval::TO_INFINITY)
            .is_none()
        {
            return 0.0;
        }

        let d_sq = (self.center - origin).length_squared();
        if d_sq <= self.radius_sq + f32::EPSILON {
            // If origin is on or inside our boundary then sample uniformly over directions
            return INV_4PI;
        }

        let cos_theta_max = (1.0 - self.radius_sq / d_sq).sqrt();
        let solid_angle = 2.0 * PI * (1.0 - cos_theta_max);

        if solid_angle <= f32::EPSILON {
            0.0
        } else {
            1.0 / solid_angle
        }
    }

    fn random_dir(&self, origin: V3, rng: &mut Rng) -> V3 {
        let d = self.center - origin;
        let d_sq = d.length_squared();

        if d_sq <= self.radius_sq + f32::EPSILON {
            // matching the condition above in pdf_value
            return random_unit_vector(rng);
        }

        Onb::new(d).transform(random_on_sphere(self.radius_sq, d_sq, rng))
    }
}
