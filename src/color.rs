use crate::{hit::Interval, v3::V3};

pub type Color = V3;

impl Color {
    pub fn ppm_string(&self) -> String {
        // Translate the [0,1] component values to the byte range [0,255].
        let intensity = Interval::new(0.0, 0.999);
        let ir = (256.0 * intensity.clamp(self.x)) as i64;
        let ig = (256.0 * intensity.clamp(self.y)) as i64;
        let ib = (256.0 * intensity.clamp(self.z)) as i64;

        format!("{ir} {ig} {ib}")
    }
}
