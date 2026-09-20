use crate::{hit::Interval, v3::V3};

/// Apply a linear to gamma transform for gamma 2
fn linear_to_gamma(linear_component: f32) -> f32 {
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

    pub const fn grey(v: f32) -> Color {
        Color::new(v, v, v)
    }

    pub fn to_rgb(&self) -> [u8; 3] {
        // Translate the [0,1] component values to the byte range [0,255].
        let intensity = Interval::new(0.0, 0.999);
        let r = (256.0 * intensity.clamp(linear_to_gamma(self.x))) as u8;
        let g = (256.0 * intensity.clamp(linear_to_gamma(self.y))) as u8;
        let b = (256.0 * intensity.clamp(linear_to_gamma(self.z))) as u8;

        [r, g, b]
    }

    pub fn ppm_string(&self) -> String {
        let [r, g, b] = self.to_rgb();

        format!("{r} {g} {b}\n")
    }
}
