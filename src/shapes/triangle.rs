use crate::{bvh::AABBox, hit::Interval, material::Material, HitRecord, Ray, P3, V3};

#[derive(Debug, Clone)]
pub struct Triangle {
    a: P3,
    ab: V3,
    ac: V3,
    normal: V3,
    unit_normal: V3,
    mat: &'static Material,
    pub bbox: AABBox,
}

impl Triangle {
    pub fn new(a: P3, b: P3, c: P3, mat: &'static Material) -> Triangle {
        let bbox1 = AABBox::new_from_points(a, b);
        let bbox2 = AABBox::new_from_points(a, c);
        let ab = b - a;
        let ac = c - a;
        let normal = ab.cross(ac);
        let unit_normal = normal.normalize();

        Self {
            a,
            ab,
            ac,
            normal,
            unit_normal,
            mat,
            bbox: AABBox::new_enclosing(bbox1, bbox2),
        }
    }

    // Calculate the intersection of a ray with a triangle using the Möller–Trumbore algorithm
    //   https://en.wikipedia.org/wiki/M%C3%B6ller%E2%80%93Trumbore_intersection_algorithm
    pub fn hits(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
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
}
