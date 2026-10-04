use crate::{
    bvh::MAX_BVH_DEPTH,
    camera::Camera,
    hit::{Hittable, Interval},
    integrator::Integrator,
    sampler::{Sampler, SimpleSampler},
    Bvh, Color, HitRecord, Ray, Rng, P3,
};
use rand::SeedableRng;
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct DepthIntegrator<C>
where
    C: Camera,
{
    pub camera: C,
    pub sampler: SimpleSampler,
    pub depth_buf: DepthBuffer,
}

impl<C> DepthIntegrator<C>
where
    C: Camera,
{
    pub fn new(camera: C, max_depth: Option<f32>) -> Self {
        let sampler = SimpleSampler::new(&camera, 0.0);

        Self {
            camera,
            sampler,
            depth_buf: DepthBuffer::new(max_depth, false),
        }
    }

    pub fn render_pass(&mut self, bvh: &Bvh, pixels: &mut Vec<Color>) {
        let (image_width, _) = self.camera.image_dims();
        let center = self.camera().center();

        self.depth_buf.resize(pixels.len());
        self.depth_buf
            .buf
            .par_chunks_mut(image_width)
            .enumerate()
            .for_each(|(j, row)| {
                let mut stack = [(0, 0.0); MAX_BVH_DEPTH];
                let mut rng = Rng::seed_from_u64(0);
                let mut r_in = Ray::default();

                for (i, px) in row.iter_mut().enumerate() {
                    let sample = self.sampler.sample_for_pixel(i, j, &mut rng);
                    self.camera.get_ray(sample, &mut r_in, &mut rng);
                    *px = DepthBuffer::pixel_depth(
                        center,
                        bvh.hits_with_stack(&r_in, Interval::TO_INFINITY, &mut stack)
                            .as_ref(),
                    );
                }
            });

        pixels.clear();
        pixels.extend(self.depth_buf.iter_pixel_colors());
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

#[derive(Debug, Clone)]
pub struct DepthBuffer {
    pub clamped_max: Option<f32>,
    pub buf: Vec<f32>,
    pub square_falloff: bool,
}

impl DepthBuffer {
    pub fn new(clamped_max: Option<f32>, square_falloff: bool) -> Self {
        Self {
            clamped_max,
            buf: Vec::new(),
            square_falloff,
        }
    }

    #[inline(always)]
    pub fn resize(&mut self, new_len: usize) {
        self.buf.resize(new_len, 0.0);
    }

    #[inline(always)]
    pub fn max_depth(&self) -> f32 {
        match self.clamped_max {
            Some(max) => max,
            None => self
                .buf
                .iter()
                .fold(0.0, |acc, d| if *d > acc { *d } else { acc }),
        }
    }

    /// Yield clamped depth values for each pixel in the depth buffer, mapped to the 0.0->1.0
    /// interval with 1.0 representing `max`.
    #[inline(always)]
    pub(super) fn iter_pixel_colors<'a>(&'a self) -> impl Iterator<Item = Color> + 'a {
        let max = self.max_depth();

        self.buf
            .iter()
            .map(move |d| Self::depth_color(*d, max, self.square_falloff))
    }

    #[inline(always)]
    pub fn pixel_depth(center: P3, hr: Option<&HitRecord>) -> f32 {
        match hr {
            Some(hr) => center.distance(hr.p),
            None => -1.0,
        }
    }

    #[inline(always)]
    pub fn depth_color(depth: f32, max: f32, square_falloff: bool) -> Color {
        Color::splat(if depth.is_sign_negative() || depth >= max {
            0.0
        } else {
            (1.0 - (depth / max)).powi(if square_falloff { 2 } else { 1 })
        })
    }

    #[inline(always)]
    pub fn pixel_color(&self, center: P3, hr: Option<&HitRecord>) -> Color {
        Self::depth_color(
            match hr {
                Some(hr) => center.distance(hr.p),
                None => -1.0,
            },
            self.max_depth(),
            self.square_falloff,
        )
    }
}
