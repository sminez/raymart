use crate::{
    bvh::{AABBox, Bvh, MAX_BVH_DEPTH},
    material::Material,
    shapes::{Quad, Sphere, Triangle},
    Ray, P3, V3,
};
use std::ops::Add;

pub mod transforms;

use transforms::{ConstantMedium, Rotate, Translate};

#[derive(Debug, Clone)]
pub enum Hittable {
    // Primitives
    Sphere(Sphere),
    Quad(Quad),
    Triangle(Triangle),
    // Compound
    List(HittableList),
    Bvh(Bvh),
    // Transforms
    ConstantMedium(ConstantMedium),
    Translate(Translate),
    Rotate(Rotate),
}

impl Hittable {
    pub fn translate(self, offset: V3) -> Hittable {
        Self::Translate(Translate::new(self, offset))
    }

    pub fn rotate(self, angle: f32) -> Hittable {
        Self::Rotate(Rotate::new(self, angle))
    }

    pub fn hits(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        match self {
            Self::Sphere(s) => s.hits(r, ray_t),
            Self::Quad(q) => q.hits(r, ray_t),
            Self::Triangle(t) => t.hits(r, ray_t),
            Self::ConstantMedium(c) => c.hits(r, ray_t),
            Self::List(l) => l.hits(r, ray_t),
            Self::Bvh(b) => b.hits(r, ray_t, &mut [0; MAX_BVH_DEPTH]),
            Self::Translate(t) => t.hits(r, ray_t),
            Self::Rotate(ro) => ro.hits(r, ray_t),
        }
    }

    pub fn bounding_box(&self) -> AABBox {
        match self {
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    pub min: f32,
    pub max: f32,
}

impl Default for Interval {
    fn default() -> Self {
        Interval::EMPTY
    }
}

impl Interval {
    pub const EMPTY: Interval = Interval::new(f32::INFINITY, -f32::INFINITY);
    pub const UNIVERSE: Interval = Interval::new(-f32::INFINITY, f32::INFINITY);
    pub const UNIT: Interval = Interval::new(0.0, 1.0);

    pub const fn new(min: f32, max: f32) -> Interval {
        Self { min, max }
    }

    pub const fn new_enclosing(a: Interval, b: Interval) -> Interval {
        Self {
            min: if a.min <= b.min { a.min } else { b.min },
            max: if a.max >= b.max { a.max } else { b.max },
        }
    }

    pub const fn size(&self) -> f32 {
        self.max - self.min
    }

    pub const fn contains(&self, x: f32) -> bool {
        self.min <= x && x <= self.max
    }

    pub const fn surrounds(&self, x: f32) -> bool {
        self.min < x && x < self.max
    }

    pub const fn clamp(&self, x: f32) -> f32 {
        if x < self.min {
            self.min
        } else if x > self.max {
            self.max
        } else {
            x
        }
    }

    #[must_use]
    pub const fn expand(&self, delta: f32) -> Interval {
        let padding = delta / 2.0;

        Interval::new(self.min - padding, self.max + padding)
    }
}

impl Add<f32> for Interval {
    type Output = Interval;

    fn add(self, rhs: f32) -> Self::Output {
        Interval::new(self.min + rhs, self.max + rhs)
    }
}

impl Add<Interval> for f32 {
    type Output = Interval;

    fn add(self, rhs: Interval) -> Self::Output {
        rhs + self
    }
}

#[derive(Debug, Copy, Clone)]
pub struct HitRecord {
    pub t: f32,
    pub p: P3,
    pub normal: V3,
    pub front_face: bool,
    pub mat: &'static Material,
    pub u: f32,
    pub v: f32,
}

impl HitRecord {
    pub fn new(
        t: f32,
        p: P3,
        outward_normal: V3,
        r: &Ray,
        mat: &'static Material,
        u: f32,
        v: f32,
    ) -> Self {
        let front_face = r.dir.dot(outward_normal) < 0.0;
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
        self.front_face = r.dir.dot(outward_normal) < 0.0;
        self.normal = if self.front_face {
            outward_normal
        } else {
            -outward_normal
        };
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
