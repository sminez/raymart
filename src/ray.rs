use crate::{P3, V3};

#[derive(Debug, Default, Clone, Copy)]
pub struct Ray {
    pub orig: P3,
    pub dir: V3,
    pub inv_dir: wide::f32x4,
    pub ro: wide::f32x4,
}

impl Ray {
    pub const fn new(orig: P3, dir: V3) -> Self {
        let ro = wide::f32x4::new([orig.x, orig.y, orig.z, 0.0]);
        let inv_dir = wide::f32x4::new([1.0 / dir.x, 1.0 / dir.y, 1.0 / dir.z, 0.0]);

        Self {
            orig,
            dir,
            inv_dir,
            ro,
        }
    }

    pub fn set(&mut self, orig: P3, dir: V3) {
        self.orig = orig;
        self.dir = dir;
        self.ro = wide::f32x4::new([orig.x, orig.y, orig.z, 0.0]);
        self.inv_dir = wide::f32x4::new([1.0 / dir.x, 1.0 / dir.y, 1.0 / dir.z, 0.0]);
    }

    pub fn at(&self, t: f32) -> P3 {
        self.orig + t * self.dir
    }
}
