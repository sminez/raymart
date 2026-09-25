use crate::{
    bvh::AABBox,
    hit::{Hittable, HittableList, Interval},
    material::Material,
    HitRecord, Ray, P3, V3,
};

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
    mat: &'static Material,
    pub bbox: AABBox,
}

impl Quad {
    pub fn new(q: P3, u: V3, v: V3, mat: &'static Material) -> Quad {
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
        }
    }

    pub fn hits(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
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
}

/// Construct a closed cuboid containing the two provided opposite vertices: a, b.
pub fn cuboid(a: P3, b: P3, mat: &'static Material) -> Hittable {
    let mut sides = HittableList::default();
    let min = a.min(b);
    let max = a.max(b);

    let dx = V3::new(max.x - min.x, 0.0, 0.0);
    let dy = V3::new(0.0, max.y - min.y, 0.0);
    let dz = V3::new(0.0, 0.0, max.z - min.z);

    sides.add(Quad::new(P3::new(min.x, min.y, max.z), dx, dy, mat).into());
    sides.add(Quad::new(P3::new(max.x, min.y, max.z), -dz, dy, mat).into());
    sides.add(Quad::new(P3::new(max.x, min.y, min.z), -dx, dy, mat).into());
    sides.add(Quad::new(P3::new(min.x, min.y, min.z), dz, dy, mat).into());
    sides.add(Quad::new(P3::new(min.x, max.y, max.z), dx, -dz, mat).into());
    sides.add(Quad::new(P3::new(min.x, min.y, min.z), dx, dz, mat).into());

    sides.into()
}
