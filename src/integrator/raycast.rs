use crate::{
    bvh::MAX_BVH_DEPTH,
    camera::Camera,
    hit::{Hittable, Interval},
    integrator::{depth::DepthBuffer, Integrator},
    material::ScatterRecord,
    sampler::{Sampler, SimpleSampler},
    Bvh, Color, HitRecord, Ray, Rng,
};
use rand::SeedableRng;
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct RaycastIntegrator<C>
where
    C: Camera,
{
    pub camera: C,
    pub sampler: SimpleSampler,
    pub depth_buf: DepthBuffer,
}

impl<C> RaycastIntegrator<C>
where
    C: Camera,
{
    pub fn new(camera: C, max_depth: f32) -> Self {
        let sampler = SimpleSampler::new(&camera, 0.0);

        Self {
            camera,
            sampler,
            depth_buf: DepthBuffer::new(Some(max_depth)),
        }
    }

    pub fn render_pass(&mut self, bvh: &Bvh, pixels: &mut [Color]) {
        let (image_width, _) = self.camera.image_dims();
        let center = self.camera().center();

        pixels
            .par_chunks_mut(image_width)
            .enumerate()
            .for_each(|(j, row)| {
                let mut stack = [(0, 0.0); MAX_BVH_DEPTH];
                let mut rng = Rng::seed_from_u64(0);
                let mut r_in = Ray::default();

                for (i, px) in row.iter_mut().enumerate() {
                    let sample = self.sampler.sample_for_pixel(i, j, &mut rng);
                    self.camera.get_ray(sample, &mut r_in, &mut rng);
                    let hr = bvh.hits_with_stack(&r_in, Interval::TO_INFINITY, &mut stack);

                    *px = self
                        .ray_color(&mut r_in, hr.as_ref(), &mut rng)
                        .unwrap_or_default()
                        * self.depth_buf.pixel_color(center, hr.as_ref());
                }
            });
    }

    fn ray_color(&self, r_in: &mut Ray, hr: Option<&HitRecord>, rng: &mut Rng) -> Option<Color> {
        let hr = hr?;

        Some(match hr.mat.scatter(r_in, hr, rng)? {
            ScatterRecord::Pdf { attenuation, .. } => attenuation,
            ScatterRecord::Skip { attenuation, .. } => attenuation,
        })
    }
}

impl<C> Integrator for RaycastIntegrator<C>
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
