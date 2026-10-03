use crate::{
    bvh::MAX_BVH_DEPTH,
    camera::Camera,
    color,
    hit::{Hittable, Interval},
    material::ScatterRecord,
    pdf::{HittablePdf, MixturePdf, Pdf},
    sampler::Sampler,
    Bvh, Color, Ray, Rng,
};
use rand::SeedableRng;
use rayon::prelude::*;
use std::{mem, ops::ControlFlow};

const RAND_JITTER: usize = 1000;

pub trait Integrator {
    fn num_iterations(&self) -> usize;
    fn image_dims(&self) -> (usize, usize);
    fn next_render_pass(
        &self,
        i: usize,
        bvh: &Bvh,
        lights: &'static dyn Hittable,
        current_pixels: &[Color],
        new_pixels: &mut Vec<Color>,
    );

    fn render(&self, bvh: Bvh, lights: &'static dyn Hittable) -> Vec<Color> {
        let (_, pixels) =
            self.render_with_hook(bvh, lights, |_, _| ControlFlow::<()>::Continue(()));

        pixels
    }

    fn render_with_hook<F, T>(
        &self,
        bvh: Bvh,
        lights: &'static dyn Hittable,
        mut per_iteration: F,
    ) -> (Option<T>, Vec<Color>)
    where
        F: FnMut(usize, &[Color]) -> ControlFlow<T>,
    {
        let (w, h) = self.image_dims();
        let mut pixels = vec![Color::default(); w * h];
        let mut new_pixels = vec![Color::default(); w * h];

        let iterations = self.num_iterations();

        for i in 1..=iterations {
            self.next_render_pass(i, &bvh, lights, &pixels, &mut new_pixels);
            mem::swap(&mut pixels, &mut new_pixels);

            if let ControlFlow::Break(break_val) = per_iteration(i, &pixels) {
                return (Some(break_val), pixels);
            }
        }

        (None, pixels)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct SimpleIntegrator<C, S>
where
    C: Camera,
    S: Sampler,
{
    pub camera: C,         // the camera being used
    pub sampler: S,        // the sampler being used
    pub samples_pp: usize, // number of random samples per pixel
    pub iterations: usize, // number of iterations with the given step size
    pub max_bounces: u8,   // maximum number of ray bounces allowed
    pub bg: Color,         // scene background color
}

impl<C, S> SimpleIntegrator<C, S>
where
    C: Camera,
    S: Sampler,
{
    pub fn new(
        camera: C,
        sampler: S,
        samples_pp: usize,
        step_size: usize,
        max_bounces: u8,
        bg: Color,
    ) -> Self {
        let (iterations, samples_pp) = if step_size > 0 && samples_pp > step_size {
            (samples_pp / step_size, step_size)
        } else {
            (1, samples_pp)
        };

        Self {
            camera,
            sampler,
            samples_pp,
            iterations,
            max_bounces,
            bg,
        }
    }

    pub fn render_pass(
        &self,
        i: usize,
        bvh: &Bvh,
        lights: &'static dyn Hittable,
        pixels: &mut [Color],
    ) {
        let (image_width, _) = self.camera.image_dims();

        pixels
            .par_chunks_mut(image_width)
            .enumerate()
            .for_each(|(j, row)| {
                let mut rng = Rng::seed_from_u64(((i * RAND_JITTER) + j) as u64);
                let mut r_in = Ray::default();
                let mut r_out = Ray::default();

                for (i, px) in row.iter_mut().enumerate() {
                    let mut acc = Color::default();

                    for _ in 0..self.samples_pp {
                        let sample = self.sampler.sample_for_pixel(i, j, &mut rng);
                        self.camera.get_ray(sample, &mut r_in, &mut rng);
                        acc += self.ray_color(&mut r_in, &mut r_out, bvh, lights, &mut rng);
                    }

                    *px = acc;
                }
            });
    }

    fn ray_color(
        &self,
        r_in: &mut Ray,
        scattered: &mut Ray,
        bvh: &Bvh,
        lights: &'static dyn Hittable,
        rng: &mut Rng,
    ) -> Color {
        let mut radiance = color::BLACK;
        let mut beta = color::WHITE;
        let mut stack = [(0, 0.0); MAX_BVH_DEPTH];

        for _ in 0..self.max_bounces {
            let hr = match bvh.hits_with_stack(r_in, Interval::TO_INFINITY, &mut stack) {
                Some(hr) => hr,
                None => return radiance + beta * self.bg,
            };

            radiance += beta * hr.mat.color_emitted(&hr);

            let (attenuation, scatter_pdf) = match hr.mat.scatter(r_in, &hr, rng) {
                Some(ScatterRecord::Pdf { attenuation, pdf }) => (attenuation, pdf),
                Some(ScatterRecord::Skip {
                    attenuation,
                    mut ray,
                }) => {
                    beta *= attenuation;
                    mem::swap(r_in, &mut ray);
                    continue;
                }
                None => break,
            };

            let light_pdf = HittablePdf::new(lights, hr.p);
            let pdf = MixturePdf {
                p1: Box::new(light_pdf),
                p2: scatter_pdf,
            };

            let dir = pdf.generate(rng);
            let dir_len_sq = dir.length_squared();
            if !dir_len_sq.is_finite() || dir_len_sq <= f32::EPSILON {
                break;
            }

            *scattered = Ray::new(hr.p, dir);
            let pdf_value = pdf.value(scattered.dir);
            if pdf_value <= 0.0 {
                break; // avoid dividing by 0 below
            }

            let scattering_pdf = hr.mat.scattering_pdf(r_in, scattered, &hr);
            beta *= attenuation * scattering_pdf / pdf_value;

            if (beta.x + beta.y + beta.z) < 0.0001 {
                break; // early exit if we can't contribute more light from here
            }

            mem::swap(r_in, scattered);
        }

        radiance
    }
}

impl<C, S> Integrator for SimpleIntegrator<C, S>
where
    C: Camera,
    S: Sampler,
{
    fn num_iterations(&self) -> usize {
        self.iterations
    }

    fn image_dims(&self) -> (usize, usize) {
        self.camera.image_dims()
    }

    fn next_render_pass(
        &self,
        i: usize,
        bvh: &Bvh,
        lights: &'static dyn Hittable,
        current_pixels: &[Color],
        new_pixels: &mut Vec<Color>,
    ) {
        let scale = 1.0 / (i * self.samples_pp) as f32;
        self.render_pass(i, bvh, lights, new_pixels);
        new_pixels.par_iter_mut().for_each(|p| *p *= scale);

        if i == 1 {
            return;
        }

        let k = (i - 1) as f32 / i as f32;
        new_pixels
            .iter_mut()
            .zip(current_pixels)
            .for_each(|(new_px, px)| *new_px += *px * k);
    }
}
