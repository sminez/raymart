use crate::{
    bvh::AABBox,
    hit::{Hittable, HittableList, Interval},
    leak_ptr,
    material::Material,
    HitRecord, Ray, Rng, P3, V3,
};
use rand::RngExt;

/// An oriented 2D quadilateral that can optionally be set to return some subregion
/// rather than the entire surface.
#[derive(Debug, Clone)]
pub struct Quad {
    q: P3,
    u: V3,
    v: V3,
    w: V3,
    normal: V3,
    d: f32,
    mat: &'static dyn Material,
    bbox: AABBox,
    area: f32,
}

impl Quad {
    pub fn new(q: P3, u: V3, v: V3, mat: &'static dyn Material) -> Quad {
        let diag1 = AABBox::new_from_points(q, q + u + v);
        let diag2 = AABBox::new_from_points(q + u, q + v);
        let bbox = AABBox::new_enclosing(diag1, diag2);

        let n = u.cross(v);
        let normal = n.normalize();
        let d = normal.dot(q);
        let w = n / n.dot(n);

        Self {
            q,
            u,
            v,
            w,
            normal,
            d,
            mat,
            bbox,
            area: n.length(),
        }
    }
}

impl Hittable for Quad {
    fn bounding_box(&self) -> AABBox {
        self.bbox
    }

    fn hits(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        let denom = self.normal.dot(r.dir);
        if denom.abs() < 1e-8 {
            return None; // ray is parallel to our plane
        }

        let t = (self.d - self.normal.dot(r.orig)) / denom;
        if !ray_t.contains(t) {
            return None; // hit point is outside of the ray interval
        }

        let intersection = r.at(t);
        let planar_hitp = intersection - self.q;
        let alpha = self.w.dot(planar_hitp.cross(self.v));
        let beta = self.w.dot(self.u.cross(planar_hitp));

        if !(Interval::UNIT.contains(alpha) && Interval::UNIT.contains(beta)) {
            return None;
        }

        Some(HitRecord::new(
            t,
            intersection,
            self.normal,
            r,
            self.mat,
            alpha,
            beta,
        ))
    }

    fn pdf_value(&self, origin: P3, dir: V3) -> f32 {
        let hr = match self.hits(&Ray::new(origin, dir), Interval::TO_INFINITY) {
            Some(hr) => hr,
            None => return 0.0,
        };

        let d_sq = dir.length_squared();
        if !d_sq.is_finite() || d_sq <= f32::EPSILON {
            return 0.0;
        }

        let cosine = (dir.dot(hr.normal) / d_sq.sqrt()).abs();
        if !cosine.is_finite() || cosine <= f32::EPSILON {
            return 0.0;
        }

        hr.t.powi(2) / (cosine * self.area)
    }

    fn random_dir(&self, origin: V3, rng: &mut Rng) -> V3 {
        let (w1, w2): (f32, f32) = rng.random();
        let p = self.q + (w1 * self.u) + (w2 * self.v);

        p - origin
    }
}

/// Construct a closed cuboid containing the two provided opposite vertices: a, b.
pub fn cuboid(a: P3, b: P3, mat: &'static dyn Material) -> HittableList {
    let mut sides = HittableList::default();
    let min = a.min(b);
    let max = a.max(b);

    let dx = V3::new(max.x - min.x, 0.0, 0.0);
    let dy = V3::new(0.0, max.y - min.y, 0.0);
    let dz = V3::new(0.0, 0.0, max.z - min.z);

    let quad = |x, y, z, u, v| Quad::new(P3::new(x, y, z), u, v, mat);

    sides.add(leak_ptr!(quad(min.x, min.y, max.z, dx, dy)));
    sides.add(leak_ptr!(quad(max.x, min.y, max.z, -dz, dy)));
    sides.add(leak_ptr!(quad(max.x, min.y, min.z, -dx, dy)));
    sides.add(leak_ptr!(quad(min.x, min.y, min.z, dz, dy)));
    sides.add(leak_ptr!(quad(min.x, max.y, max.z, dx, -dz)));
    sides.add(leak_ptr!(quad(min.x, min.y, min.z, dx, dz)));

    sides
}
