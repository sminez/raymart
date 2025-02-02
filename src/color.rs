use crate::v3::V3;

pub type Color = V3;

impl Color {
    pub fn ppm_string(&self) -> String {
        // Translate the [0,1] component values to the byte range [0,255].
        let ir = (255.999 * self.x) as i64;
        let ig = (255.999 * self.y) as i64;
        let ib = (255.999 * self.z) as i64;

        format!("{ir} {ig} {ib}")
    }
}
