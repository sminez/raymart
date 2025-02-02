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

    pub fn color(&self) -> Color {
        let unit_dir = self.dir.unit_vector();
        let a = 0.5 * (unit_dir.y + 1.0);

        (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
    }
}
