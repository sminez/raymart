use crate::{camera::SimpleCamera, p, v, P3, V3};
use serde::Deserialize;

const DEFAULT_FOCUS_DIST: f32 = 10.0;

#[derive(Debug, Clone, Deserialize)]
pub struct CameraSpec {
    pub image_width: usize,
    pub aspect_ratio: f32,
    pub fov: f32,
    pub from: [f32; 3],
    pub at: [f32; 3],
    pub v_up: [f32; 3],
    #[serde(default)]
    pub defocus_angle: f32,
    #[serde(default)]
    pub focus_dist: Option<f32>,
}

impl CameraSpec {
    pub fn into_simple_camera(self) -> SimpleCamera {
        let v_up = v!(self.v_up[0], self.v_up[1], self.v_up[2]);
        let look_from = p!(self.from[0], self.from[1], self.from[2]);
        let look_at = p!(self.at[0], self.at[1], self.at[2]);

        SimpleCamera::new(
            self.aspect_ratio,
            self.image_width,
            self.fov,
            look_from,
            look_at,
            v_up,
            self.defocus_angle,
            self.focus_dist.unwrap_or(DEFAULT_FOCUS_DIST),
        )
    }
}
