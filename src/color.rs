use std::ops::{Deref, DerefMut};

use crate::v3::V3;

#[derive(Debug, Default, Copy, Clone)]
pub struct Color(pub V3);

impl Color {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self(V3 { x, y, z })
    }

    pub fn print_ppm(&self) {
        // Translate the [0,1] component values to the byte range [0,255].
        let ir = (255.999 * self.x) as i64;
        let ig = (255.999 * self.y) as i64;
        let ib = (255.999 * self.z) as i64;

        println!("{ir} {ig} {ib}");
    }
}

impl From<V3> for Color {
    fn from(value: V3) -> Self {
        Self(value)
    }
}

impl Deref for Color {
    type Target = V3;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Color {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
