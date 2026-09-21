use crate::{hit::Interval, V3};

/// Apply a linear to gamma transform for gamma 2
fn linear_to_gamma(linear_component: f32) -> f32 {
    if linear_component > 0.0 {
        linear_component.sqrt()
    } else {
        0.0
    }
}

pub type Color = V3;

pub const WHITE: Color = Color::new(1.0, 1.0, 1.0);
pub const BLACK: Color = Color::new(0.0, 0.0, 0.0);

pub fn to_rgb(c: Color) -> [u8; 3] {
    // Translate the [0,1] component values to the byte range [0,255].
    let intensity = Interval::new(0.0, 0.999);
    let r = (256.0 * intensity.clamp(linear_to_gamma(c.x))) as u8;
    let g = (256.0 * intensity.clamp(linear_to_gamma(c.y))) as u8;
    let b = (256.0 * intensity.clamp(linear_to_gamma(c.z))) as u8;

    [r, g, b]
}

pub fn ppm_string(c: Color) -> String {
    let [r, g, b] = to_rgb(c);

    format!("{r} {g} {b}\n")
}
