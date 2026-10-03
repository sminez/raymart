use crate::{
    bvh::MAX_BVH_DEPTH,
    camera::Camera,
    hit::{Hittable, Interval},
    integrator::Integrator,
    sampler::{Sampler, SimpleSampler},
    Bvh, Color, Ray, Rng,
};
use rand::SeedableRng;
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct DepthIntegrator<C>
where
    C: Camera,
{
    pub camera: C,              // the camera being used
    pub sampler: SimpleSampler, // a simple sampler that targets each pixel directly
    pub depth_buf: Vec<f32>,    // buffer for storing depth values
}

impl<C> DepthIntegrator<C>
where
    C: Camera,
{
    pub fn new(camera: C) -> Self {
        let sampler = SimpleSampler::new(&camera, 0.0);

        Self {
            camera,
            sampler,
            depth_buf: Vec::new(),
        }
    }

    pub fn render_pass(&mut self, bvh: &Bvh, pixels: &mut [Color]) {
        let (image_width, _) = self.camera.image_dims();
        self.depth_buf.resize(pixels.len(), 0.0);

        self.depth_buf
            .par_chunks_mut(image_width)
            .enumerate()
            .for_each(|(j, row)| {
                let mut stack = [(0, 0.0); MAX_BVH_DEPTH];
                let mut rng = Rng::seed_from_u64(0);
                let mut r_in = Ray::default();

                for (i, px) in row.iter_mut().enumerate() {
                    let sample = self.sampler.sample_for_pixel(i, j, &mut rng);
                    self.camera.get_ray(sample, &mut r_in, &mut rng);
                    let dist = match bvh.hits_with_stack(&r_in, Interval::TO_INFINITY, &mut stack) {
                        Some(hr) => self.camera.center().distance(hr.p),
                        None => -1.0,
                    };

                    *px = dist;
                }
            });

        let max = self
            .depth_buf
            .iter()
            .fold(0.0, |acc, d| if *d > acc { *d } else { acc });

        pixels
            .iter_mut()
            .zip(self.depth_buf.iter())
            .for_each(|(px, d)| {
                *px = Color::splat(if d.is_sign_negative() {
                    0.0
                } else {
                    (max - *d) / max
                })
            });
    }
}

impl<C> Integrator for DepthIntegrator<C>
where
    C: Camera,
{
    fn num_iterations(&self) -> usize {
        1
    }

    fn camera(&self) -> &dyn Camera {
        &self.camera
    }

    fn next_render_pass(
        &mut self,
        _i: usize,
        bvh: &Bvh,
        _lights: &'static dyn Hittable,
        _current_pixels: &[Color],
        new_pixels: &mut Vec<Color>,
    ) {
        self.render_pass(bvh, new_pixels);
    }
}
