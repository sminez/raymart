use crate::{Point3, Ray, V3};

#[derive(Debug, Clone, Copy)]
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

    pub const fn new(min: f64, max: f64) -> Self {
        Self { min, max }
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
}

#[derive(Debug, Default, Clone, Copy)]
pub struct HitRecord {
    pub t: f64,
    pub p: Point3,
    pub normal: V3,
    pub front_face: bool,
}

impl HitRecord {
    pub fn new(t: f64, p: Point3, outward_normal: V3, r: &Ray) -> Self {
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

pub trait Hittable: Send + Sync + 'static {
    fn hits(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord>;
}

#[derive(Default)]
pub struct HittableList {
    objects: Vec<Box<dyn Hittable>>,
}

impl HittableList {
    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn add(&mut self, obj: impl Hittable + 'static) {
        self.objects.push(Box::new(obj));
    }
}

impl Hittable for HittableList {
    fn hits(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
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

#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    center: Point3,
    radius: f64,
    radius_sq: f64,
}

impl Sphere {
    pub fn new(center: Point3, radius: f64) -> Self {
        let r = radius.max(0.0);

        Self {
            center,
            radius: r,
            radius_sq: r * r,
        }
    }
}

impl Hittable for Sphere {
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
        let mut root = (h - sqrt_disc) / a;
        if !ray_t.surrounds(root) {
            root = (h + sqrt_disc) / a;
            if !ray_t.surrounds(root) {
                return None;
            }
        }

        let p = r.at(root);
        let outward_normal = (p - self.center) / self.radius;

        Some(HitRecord::new(root, p, outward_normal, r))
    }
}
