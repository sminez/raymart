use crate::{hit::Interval, v3::V3};

/// Apply a linear to gamma transform for gamma 2
fn linear_to_gamma(linear_component: f64) -> f64 {
    if linear_component > 0.0 {
        linear_component.sqrt()
    } else {
        0.0
    }
}

pub type Color = V3;

impl Color {
    pub const WHITE: Color = Color::new(1.0, 1.0, 1.0);
    pub const BLACK: Color = Color::new(0.0, 0.0, 0.0);

    pub const fn grey(v: f64) -> Color {
        Color::new(v, v, v)
    }

    pub fn ppm_string(&self) -> String {
        // Translate the [0,1] component values to the byte range [0,255].
        let intensity = Interval::new(0.0, 0.999);
        let ir = (256.0 * intensity.clamp(linear_to_gamma(self.x))) as i64;
        let ig = (256.0 * intensity.clamp(linear_to_gamma(self.y))) as i64;
        let ib = (256.0 * intensity.clamp(linear_to_gamma(self.z))) as i64;

        format!("{ir} {ig} {ib}")
    }
}
