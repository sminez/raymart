use rand::random_range;

use crate::{
    bbox::{AABBox, BvhNode},
    material::{Material, Texture},
    Color, Ray, P3, V3,
};
use std::{f64::consts::PI, ops::Add};

const INV_PI: f64 = 1.0 / PI;
const INV_2PI: f64 = 1.0 / (2.0 * PI);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    pub min: f64,
    pub max: f64,
}

impl Default for Interval {
    fn default() -> Self {
        Interval::EMPTY
    }
}

impl Interval {
    pub const EMPTY: Interval = Interval::new(f64::INFINITY, -f64::INFINITY);
    pub const UNIVERSE: Interval = Interval::new(-f64::INFINITY, f64::INFINITY);
    pub const UNIT: Interval = Interval::new(0.0, 1.0);

    pub const fn new(min: f64, max: f64) -> Interval {
        Self { min, max }
    }

    pub const fn new_enclosing(a: Interval, b: Interval) -> Interval {
        Self {
            min: if a.min <= b.min { a.min } else { b.min },
            max: if a.max >= b.max { a.max } else { b.max },
        }
    }

    pub const fn size(&self) -> f64 {
        self.max - self.min
    }

    pub const fn contains(&self, x: f64) -> bool {
        self.min <= x && x <= self.max
    }

    pub const fn surrounds(&self, x: f64) -> bool {
        self.min < x && x < self.max
    }

    pub const fn clamp(&self, x: f64) -> f64 {
        if x < self.min {
            self.min
        } else if x > self.max {
            self.max
        } else {
            x
        }
    }

    #[must_use]
    pub const fn expand(&self, delta: f64) -> Interval {
        let padding = delta / 2.0;

        Interval::new(self.min - padding, self.max + padding)
    }
}

impl Add<f64> for Interval {
    type Output = Interval;

    fn add(self, rhs: f64) -> Self::Output {
        Interval::new(self.min + rhs, self.max + rhs)
    }
}

impl Add<Interval> for f64 {
    type Output = Interval;

    fn add(self, rhs: Interval) -> Self::Output {
        rhs + self
    }
}

#[derive(Debug, Clone)]
pub struct HitRecord {
    pub t: f64,
    pub p: P3,
    pub normal: V3,
    pub front_face: bool,
    pub mat: Material,
    pub u: f64,
    pub v: f64,
}

impl HitRecord {
    pub fn new(t: f64, p: P3, outward_normal: V3, r: &Ray, mat: Material, u: f64, v: f64) -> Self {
        let front_face = r.dir.dot(&outward_normal) < 0.0;
        let normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };

        Self {
            t,
            p,
            normal,
            front_face,
            mat,
            u,
            v,
        }
    }

    /// Sets the [HitRecord] normal vector.
    ///
    /// `outward_normal` is assumed to be of unit length.
    pub fn set_face_normal(&mut self, r: &Ray, outward_normal: V3) {
        self.front_face = r.dir.dot(&outward_normal) < 0.0;
        self.normal = if self.front_face {
            outward_normal
        } else {
            -outward_normal
        };
    }
}

#[derive(Debug, Clone)]
pub enum Hittable {
    // Primatives
    Empty,
    Sphere(Sphere),
    Quad(Quad),
    Triangle(Triangle),
    ConstantMedium(ConstantMedium),
    // Compound
    List(HittableList),
    Bvh(&'static BvhNode),
    // Transforms
    Translate(Translate),
    Rotate(Rotate),
}

impl Hittable {
    pub fn translate(self, offset: V3) -> Hittable {
        Self::Translate(Translate::new(self, offset))
    }

    pub fn rotate(self, angle: f64) -> Hittable {
        Self::Rotate(Rotate::new(self, angle))
    }

    pub fn hits(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        match self {
            Self::Empty => None,
            Self::Sphere(s) => s.hits(r, ray_t),
            Self::Quad(q) => q.hits(r, ray_t),
            Self::Triangle(t) => t.hits(r, ray_t),
            Self::ConstantMedium(c) => c.hits(r, ray_t),
            Self::List(l) => l.hits(r, ray_t),
            Self::Bvh(b) => b.hits(r, ray_t),
            Self::Translate(t) => t.hits(r, ray_t),
            Self::Rotate(ro) => ro.hits(r, ray_t),
        }
    }

    pub fn bounding_box(&self) -> AABBox {
        match self {
            Self::Empty => AABBox::EMPTY,
            Self::Sphere(s) => s.bbox,
            Self::Quad(q) => q.bbox,
            Self::Triangle(t) => t.bbox,
            Self::ConstantMedium(c) => c.bounding_box(),
            Self::List(l) => l.bbox,
            Self::Bvh(b) => b.bbox,
            Self::Translate(t) => t.bbox,
            Self::Rotate(r) => r.bbox,
        }
    }
}

impl From<Sphere> for Hittable {
    fn from(s: Sphere) -> Self {
        Self::Sphere(s)
    }
}

impl From<Quad> for Hittable {
    fn from(q: Quad) -> Self {
        Self::Quad(q)
    }
}

impl From<Triangle> for Hittable {
    fn from(t: Triangle) -> Self {
        Self::Triangle(t)
    }
}

impl From<ConstantMedium> for Hittable {
    fn from(c: ConstantMedium) -> Self {
        Self::ConstantMedium(c)
    }
}

impl From<HittableList> for Hittable {
    fn from(l: HittableList) -> Self {
        Self::List(l)
    }
}

#[derive(Default, Debug, Clone)]
pub struct HittableList {
    pub objects: Vec<Hittable>,
    bbox: AABBox,
}

impl HittableList {
    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn add(&mut self, obj: Hittable) {
        self.bbox = AABBox::new_enclosing(self.bbox, obj.bounding_box());
        self.objects.push(obj);
    }

    pub fn hits(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        let mut rec: Option<HitRecord> = None;
        let mut closest_so_far = ray_t.max;
        for obj in self.objects.iter() {
            if let Some(obj_rec) = obj.hits(r, Interval::new(ray_t.min, closest_so_far)) {
                closest_so_far = obj_rec.t;
                rec = Some(obj_rec);
            }
        }

        rec
    }
}

#[derive(Debug, Clone)]
pub struct Sphere {
    center: P3,
    inv_radius: f64,
    radius_sq: f64,
    mat: Material,
    bbox: AABBox,
}

impl Sphere {
    pub fn new(center: P3, radius: f64, mat: Material) -> Self {
        let r = radius.max(0.0);
        let rvec = V3::new(r, r, r);
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
    fn hits(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        let oc = self.center - r.orig;

        let a = r.dir.square_length();
        let h = r.dir.dot(&oc);
        let c = oc.square_length() - self.radius_sq;
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

        let theta = (-outward_normal.y).acos();
        let phi = (-outward_normal.z).atan2(outward_normal.x) + PI;
        let u = phi * INV_2PI;
        let v = theta * INV_PI;

        Some(HitRecord::new(root, p, outward_normal, r, self.mat, u, v))
    }
}

#[derive(Debug, Clone)]
pub struct Triangle {
    a: P3,
    ab: V3,
    ac: V3,
    normal: V3,
    mat: Material,
    pub bbox: AABBox,
}

impl Triangle {
    pub fn new(a: P3, b: P3, c: P3, mat: Material) -> Triangle {
        let bbox1 = AABBox::new_from_points(a, b);
        let bbox2 = AABBox::new_from_points(a, c);
        let ab = b - a;
        let ac = c - a;
        let normal = ab.cross(&ac);

        Self {
            a,
            ab,
            ac,
            normal,
            mat,
            bbox: AABBox::new_enclosing(bbox1, bbox2),
        }
    }

    // Calculate the intersection of a ray with a triangle using the Möller–Trumbore algorithm
    //   https://en.wikipedia.org/wiki/M%C3%B6ller%E2%80%93Trumbore_intersection_algorithm
    pub fn hits(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        // If r . normal is 0 then the ray is parallel to the triangle plane and no hit is possible
        let det = -(r.dir.dot(&self.normal));
        if det.abs() < 1e-8 {
            return None;
        }

        let inv_det = 1.0 / det;
        let ao = r.orig - self.a;
        let r_x_ao = ao.cross(&r.dir);

        // hit point needs to be contained by the ray interval
        let t = ao.dot(&self.normal) * inv_det;
        if !ray_t.surrounds(t) {
            return None;
        }

        // barycentric coords of the intersection point
        //   https://en.wikipedia.org/wiki/Barycentric_coordinate_system
        let u = self.ac.dot(&r_x_ao) * inv_det;
        let v = -self.ab.dot(&r_x_ao) * inv_det;
        if u < 0.0 || v < 0.0 || u + v > 1.0 {
            return None;
        }

        let p = r.at(t);

        Some(HitRecord::new(t, p, self.normal, r, self.mat, u, v))
    }
}

/// An oriented 2D quadilateral that can optionally be set to return some subregion
/// rather than the entire surface.
#[derive(Debug, Clone)]
pub struct Quad {
    q: P3,
    u: V3,
    v: V3,
    w: V3,
    normal: V3,
    d: f64,
    mat: Material,
    shape: QuadShape,
    bbox: AABBox,
}

impl Quad {
    pub fn new(q: P3, u: V3, v: V3, mat: Material) -> Quad {
        Self::new_with_shape(q, u, v, mat, QuadShape::Quad)
    }

    pub fn new_triangle(q: P3, u: V3, v: V3, mat: Material) -> Quad {
        Self::new_with_shape(q, u, v, mat, QuadShape::Triangle)
    }

    /// Radius needs to be 0..1
    pub fn new_disk(q: P3, u: V3, v: V3, r: f64, mat: Material) -> Quad {
        let shape = QuadShape::Disk {
            r2: (r * 0.5).powi(2),
        };

        Self::new_with_shape(q, u, v, mat, shape)
    }

    /// Radii needs to be 0..1
    pub fn new_ring(q: P3, u: V3, v: V3, r1: f64, r2: f64, mat: Material) -> Quad {
        let shape = QuadShape::Ring {
            r1_2: (r1 * 0.5).powi(2),
            r2_2: (r2 * 0.5).powi(2),
        };

        Self::new_with_shape(q, u, v, mat, shape)
    }

    fn new_with_shape(q: P3, u: V3, v: V3, mat: Material, shape: QuadShape) -> Quad {
        let diag1 = AABBox::new_from_points(q, q + u + v);
        let diag2 = AABBox::new_from_points(q + u, q + v);
        let bbox = AABBox::new_enclosing(diag1, diag2);

        let n = u.cross(&v);
        let normal = n.unit_vector();
        let d = normal.dot(&q);
        let w = n / n.dot(&n);

        Self {
            q,
            u,
            v,
            w,
            normal,
            d,
            mat,
            shape,
            bbox,
        }
    }

    fn hits(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        let denom = self.normal.dot(&r.dir);
        if denom.abs() < 1e-8 {
            return None; // ray is parallel to our plane
        }

        let t = (self.d - self.normal.dot(&r.orig)) / denom;
        if !ray_t.contains(t) {
            return None; // hit point is outside of the ray interval
        }

        let intersection = r.at(t);
        let planar_hitp = intersection - self.q;
        let alpha = self.w.dot(&planar_hitp.cross(&self.v));
        let beta = self.w.dot(&self.u.cross(&planar_hitp));

        if !self.shape.hits_surface(alpha, beta) {
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

#[derive(Debug, Clone, Copy)]
enum QuadShape {
    Quad,
    Triangle,
    Disk { r2: f64 },
    Ring { r1_2: f64, r2_2: f64 },
}

impl QuadShape {
    // u,v surface coordinates are [0,1]x[1,0]
    fn hits_surface(&self, alpha: f64, beta: f64) -> bool {
        match self {
            Self::Quad => Interval::UNIT.contains(alpha) && Interval::UNIT.contains(beta),
            Self::Disk { r2 } => (alpha - 0.5).powi(2) + (beta - 0.5).powi(2) < *r2,
            Self::Ring { r1_2, r2_2 } => {
                let p = (alpha - 0.5).powi(2) + (beta - 0.5).powi(2);
                p > *r2_2 && p < *r1_2
            }
            Self::Triangle => alpha > 0. && beta > 0. && alpha + beta < 1.,
        }
    }
}

/// Construct a closed cuboid containing the two provided opposite vertices: a, b.
pub fn cuboid(a: P3, b: P3, mat: Material) -> Hittable {
    let mut sides = HittableList::default();
    let min = P3::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z));
    let max = P3::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z));

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

#[derive(Debug, Clone)]
pub struct ConstantMedium {
    boundary: &'static Hittable,
    neg_inv_density: f64,
    phase_func: Material,
}

impl ConstantMedium {
    pub fn new(boundary: Hittable, density: f64, color: Color) -> ConstantMedium {
        Self::new_with_texture(boundary, density, Texture::solid(color))
    }

    pub fn new_with_texture(boundary: Hittable, density: f64, texture: Texture) -> ConstantMedium {
        let neg_inv_density = -1.0 / density;

        Self {
            boundary: Box::leak(Box::new(boundary)),
            neg_inv_density,
            phase_func: Material::isotropic_texture(texture),
        }
    }

    pub fn bounding_box(&self) -> AABBox {
        self.boundary.bounding_box()
    }

    pub fn hits(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        let mut hr1 = self.boundary.hits(r, Interval::UNIVERSE)?;
        let i2 = Interval::new(hr1.t + 0.0001, f64::INFINITY);
        let mut hr2 = self.boundary.hits(r, i2)?;

        hr1.t = hr1.t.max(ray_t.min);
        hr2.t = hr2.t.min(ray_t.max);
        if hr1.t > hr2.t {
            return None;
        }

        hr1.t = hr1.t.max(0.0);

        let r_len = r.dir.length();
        let dist_in_boundary = (hr2.t - hr1.t) * r_len;
        let hit_dist = self.neg_inv_density * random_range(0.0..1.0f64).log2();
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
    bbox: AABBox,
}

impl Translate {
    fn new(inner: Hittable, offset: V3) -> Translate {
        let bbox = inner.bounding_box() + offset;

        Self {
            inner: Box::new(inner),
            offset,
            bbox,
        }
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
}

/// Rotation around y
#[derive(Debug, Clone)]
pub struct Rotate {
    inner: Box<Hittable>,
    sin_theta: f64,
    cos_theta: f64,
    bbox: AABBox,
}

impl Rotate {
    fn new(inner: Hittable, angle: f64) -> Rotate {
        let rad = angle.to_radians();
        let sin_theta = rad.sin();
        let cos_theta = rad.cos();
        let bbox = inner.bounding_box();

        let mut min = P3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
        let mut max = P3::new(-f64::INFINITY, -f64::INFINITY, -f64::INFINITY);

        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let x = i as f64 * bbox.x.max + (1 - i) as f64 * bbox.x.min;
                    let y = j as f64 * bbox.y.max + (1 - j) as f64 * bbox.y.min;
                    let z = k as f64 * bbox.z.max + (1 - k) as f64 * bbox.z.min;

                    let new_x = cos_theta * x + sin_theta * z;
                    let new_z = -sin_theta * x + cos_theta * z;
                    let v = V3::new(new_x, y, new_z);

                    for c in 0..3 {
                        min[c] = min[c].min(v[c]);
                        max[c] = max[c].max(v[c]);
                    }
                }
            }
        }

        let bbox = AABBox::new_from_points(min, max);

        Self {
            inner: Box::new(inner),
            sin_theta,
            cos_theta,
            bbox,
        }
    }

    #[inline]
    fn rot_f(&self, v_in: V3) -> V3 {
        V3::new(
            self.cos_theta * v_in.x - self.sin_theta * v_in.z,
            v_in.y,
            self.sin_theta * v_in.x + self.cos_theta * v_in.z,
        )
    }

    #[inline]
    fn rot_b(&self, v_in: V3) -> V3 {
        V3::new(
            self.cos_theta * v_in.x + self.sin_theta * v_in.z,
            v_in.y,
            -self.sin_theta * v_in.x + self.cos_theta * v_in.z,
        )
    }

    fn hits(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        // Transform the ray from world space to object space.
        let rot_r = Ray::new(self.rot_f(r.orig), self.rot_f(r.dir));

        // If the rotated ray hits...
        let mut hr = self.inner.hits(&rot_r, ray_t)?;

        // apply the rotation to the hit record and return
        hr.p = self.rot_b(hr.p);
        hr.normal = self.rot_b(hr.normal);

        Some(hr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_test_case::test_case;

    #[test_case(Interval::new(1.0, 2.0), Interval::new(1.0, 2.0), Interval::new(1.0, 2.0); "idempotent")]
    #[test_case(Interval::new(1.0, 3.0), Interval::new(2.0, 5.0), Interval::new(1.0, 5.0); "overlapping")]
    #[test_case(Interval::new(1.0, 2.0), Interval::new(3.0, 5.0), Interval::new(1.0, 5.0); "disjoint")]
    #[test_case(Interval::EMPTY, Interval::new(3.0, 5.0), Interval::new(3.0, 5.0); "with empty")]
    #[test_case(Interval::UNIVERSE, Interval::new(3.0, 5.0), Interval::UNIVERSE; "with universe")]
    #[test]
    fn enclosing_works(a: Interval, b: Interval, expected: Interval) {
        let res = Interval::new_enclosing(a, b);

        assert_eq!(res, expected);
    }
}
