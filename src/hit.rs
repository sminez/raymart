use crate::{bbox::AABBox, material::Material, Ray, P3, V3};
use std::f64::consts::PI;

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

    pub const fn expand(&self, delta: f64) -> Interval {
        let padding = delta / 2.0;

        Interval::new(self.min - padding, self.max + padding)
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
    Empty,
    Sphere(Sphere),
    List(HittableList),
}

impl Hittable {
    pub fn hits(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        match self {
            Self::Empty => None,
            Self::Sphere(s) => s.hits(r, ray_t),
            Self::List(l) => l.hits(r, ray_t),
        }
    }

    pub fn bounding_box(&self) -> AABBox {
        match self {
            Self::Empty => AABBox::EMPTY,
            Self::Sphere(s) => s.bbox,
            Self::List(l) => l.bbox,
        }
    }
}

impl From<Sphere> for Hittable {
    fn from(s: Sphere) -> Self {
        Self::Sphere(s)
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
    center: Ray,
    radius: f64,
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
            center: Ray::new(center, V3::default(), 0.0),
            radius: r,
            radius_sq: r * r,
            mat,
            bbox,
        }
    }

    pub fn new_moving(center1: P3, center2: P3, radius: f64, mat: Material) -> Self {
        let r = radius.max(0.0);
        let rvec = V3::new(r, r, r);
        let center = Ray::new(center1, center2 - center1, 0.0);
        let bbox1 = AABBox::new_from_points(center.at(0.0) - rvec, center.at(0.0) + rvec);
        let bbox2 = AABBox::new_from_points(center.at(1.0) - rvec, center.at(1.0) + rvec);
        let bbox = AABBox::new_enclosing(bbox1, bbox2);

        Self {
            center,
            radius: r,
            radius_sq: r * r,
            mat,
            bbox,
        }
    }

    /// The derivation of the calculation here is given in section 5 of Ray tracing in one weekend
    /// https://raytracing.github.io/books/RayTracingInOneWeekend.html
    fn hits(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        let current_center = self.center.at(r.time);
        let oc = current_center - r.orig;

        let a = r.dir.square_length();
        let h = r.dir.dot(&oc);
        let c = oc.square_length() - self.radius_sq;
        let discriminant = h * h - a * c;

        if discriminant < 0.0 {
            return None;
        }

        let sqrt_disc = discriminant.sqrt();

        // Find the nearest root that lies between tmin & tmax
        let mut root = (h - sqrt_disc) / a;
        if !ray_t.surrounds(root) {
            root = (h + sqrt_disc) / a;
            if !ray_t.surrounds(root) {
                return None;
            }
        }

        let p = r.at(root);
        let outward_normal = (p - current_center) / self.radius;

        let theta = (-outward_normal.y).acos();
        let phi = (-outward_normal.z).atan2(outward_normal.x) + PI;
        let u = phi / (2.0 * PI);
        let v = theta / PI;

        Some(HitRecord::new(
            root,
            p,
            outward_normal,
            r,
            self.mat.clone(),
            u,
            v,
        ))
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
