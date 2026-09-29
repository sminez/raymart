use crate::{
    bvh::AABBox,
    hit::{Hittable, Interval},
    material::Material,
    HitRecord, Ray, Rng, P3, V3,
};
use rand::RngExt;

#[derive(Debug, Clone)]
pub struct Triangle {
    a: P3,
    ab: V3,
    ac: V3,
    normal: V3,
    unit_normal: V3,
    area: f32,
    mat: &'static dyn Material,
    bbox: AABBox,
}

impl Triangle {
    pub fn new(a: P3, b: P3, c: P3, mat: &'static dyn Material) -> Triangle {
        let bbox1 = AABBox::new_from_points(a, b);
        let bbox2 = AABBox::new_from_points(a, c);
        let ab = b - a;
        let ac = c - a;
        let normal = ab.cross(ac);
        let unit_normal = normal.normalize();
        // normal = ab.cross(ac) so its length is the parallelogram area, so the triangle area is
        // just half the length
        let area = 0.5 * normal.length();

        Self {
            a,
            ab,
            ac,
            normal,
            unit_normal,
            area,
            mat,
            bbox: AABBox::new_enclosing(bbox1, bbox2),
        }
    }
}

impl Hittable for Triangle {
    fn bounding_box(&self) -> AABBox {
        self.bbox
    }

    // Calculate the intersection of a ray with a triangle using the Möller–Trumbore algorithm
    //   https://en.wikipedia.org/wiki/M%C3%B6ller%E2%80%93Trumbore_intersection_algorithm
    fn hits(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        // If r . normal is 0 then the ray is parallel to the triangle plane and no hit is possible
        let det = -(r.dir.dot(self.normal));
        if det.abs() < 1e-8 {
            return None;
        }

        let inv_det = 1.0 / det;
        let ao = r.orig - self.a;
        let r_x_ao = ao.cross(r.dir);

        // hit point needs to be contained by the ray interval
        let t = ao.dot(self.normal) * inv_det;
        if !ray_t.surrounds(t) {
            return None;
        }

        // barycentric coords of the intersection point
        //   https://en.wikipedia.org/wiki/Barycentric_coordinate_system
        let u = self.ac.dot(r_x_ao) * inv_det;
        let v = -self.ab.dot(r_x_ao) * inv_det;
        if u < 0.0 || v < 0.0 || u + v > 1.0 {
            return None;
        }

        let p = r.at(t);

        Some(HitRecord::new(t, p, self.unit_normal, r, self.mat, u, v))
    }

    fn pdf_value(&self, origin: P3, dir: V3) -> f32 {
        let hr = match self.hits(&Ray::new(origin, dir), Interval::TO_INFINITY) {
            Some(hr) => hr,
            None => return 0.0,
        };
        let d_sq = dir.length_squared();
        let cosine = (dir.dot(hr.normal) / d_sq.sqrt()).abs();

        hr.t.powi(2) / (cosine * self.area)
    }

    fn random_dir(&self, origin: V3, rng: &mut Rng) -> V3 {
        let (r1, r2): (f32, f32) = rng.random();
        let sr1 = r1.sqrt();
        let u = 1.0 - sr1;
        let v = r2 * sr1;
        let p = self.a + u * self.ab + v * self.ac;

        p - origin
    }
}
