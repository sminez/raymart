use crate::{camera::Camera, v3::Onb, Rng, P3, V3};
use rand::RngExt;

pub trait Sampler: Send + Sync {
    fn sample_for_pixel(&self, i: usize, j: usize, rng: &mut Rng) -> V3;
}

#[derive(Debug, Copy, Clone)]
pub struct SimpleSampler {
    origin: P3,  // location of pixel 0,0
    delta_x: V3, // offset to pixel to the right
    delta_y: V3, // offset to pixel below
}

impl SimpleSampler {
    pub fn new(camera: &impl Camera) -> Self {
        let Onb { u, v, .. } = camera.basis();
        let (viewport_width, viewport_height) = camera.viewport_dims();
        let (image_width, image_height) = camera.image_dims();

        let viewport_u = viewport_width * u;
        let viewport_v = viewport_height * -v;

        let delta_x = viewport_u / image_width as f32;
        let delta_y = viewport_v / image_height as f32;
        let origin = camera.viewport_top_left() + 0.5 * (delta_x + delta_y);

        Self {
            origin,
            delta_x,
            delta_y,
        }
    }
}

impl Sampler for SimpleSampler {
    fn sample_for_pixel(&self, i: usize, j: usize, rng: &mut Rng) -> V3 {
        // Vector to a random point in the [-.5,-.5]-[+.5,+.5] unit square
        let offset = V3::new(
            rng.random_range(-0.5..0.5),
            rng.random_range(-0.5..0.5),
            0.0,
        );

        self.origin
            + ((i as f32 + offset.x) * self.delta_x)
            + ((j as f32 + offset.y) * self.delta_y)
    }
}
