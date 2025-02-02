use crate::{
    v3::{Point3, V3},
    Color,
};

pub struct Ray {
    pub orig: Point3,
    pub dir: V3,
}

impl Ray {
    pub const fn new(orig: Point3, dir: V3) -> Self {
        Self { orig, dir }
    }

    pub fn at(&self, t: f64) -> Point3 {
        self.orig + t * self.dir
    }

    /// The derivation of the calculation here is given in section 5 of Ray tracing in one weekend
    /// https://raytracing.github.io/books/RayTracingInOneWeekend.html
    fn hits_sphere(&self, center: &Point3, radius: f64) -> f64 {
        let oc = *center - self.orig;

        let a = self.dir.square_length();
        let b = -2.0 * self.dir.dot(&oc);
        let c = oc.square_length() - radius * radius;
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            -1.0
        } else {
            (-b - discriminant.sqrt()) / (2.0 * a)
        }
    }

    pub fn color(&self) -> Color {
        let t = self.hits_sphere(&V3::new(0.0, 0.0, -1.0), 0.5);
        if t > 0.0 {
            let n = self.at(t) - V3::new(0.0, 0.0, -1.0);
            return 0.5 * Color::new(n.x + 1.0, n.y + 1.0, n.z + 1.0);
        }

        let unit_dir = self.dir.unit_vector();
        let a = 0.5 * (unit_dir.y + 1.0);

        (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
    }
}
