use crate::{
    bvh::{Bvh, MAX_BVH_DEPTH},
    color,
    hit::Interval,
    sdl::MainThreadState,
    v3, Backend, Color, P3, V3,
};
use rand::random_range;
use rayon::prelude::*;
use sdl2::{event::Event, keyboard::Keycode};
use std::{cmp::max, fs, mem, time::Instant};

#[derive(Debug, Copy, Clone)]
pub struct Camera {
    image_width: u16,   // rendered image width (pixels)
    image_height: u16,  // rendered image height (pixels)
    samples_pp: u16,    // number of random samples per pixel
    iterations: u16,    // number of iterations with the given step size
    max_bounces: u8,    // maximum number of ray bounces allowed
    bg: Color,          // scene background color
    center: P3,         // camera center
    pixel_origin: P3,   // location of pixel 0,0
    pixel_delta_u: V3,  // offset to pixel to the right
    pixel_delta_v: V3,  // offset to pixel below
    defocus_angle: f32, // angle of the defocus disk
    defocus_disk_u: V3, // defocus disk horizontal radius
    defocus_disk_v: V3, // defocus disk vertical radius
}

impl Camera {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        aspect_ratio: f32,
        image_width: u16,
        samples_pp: u16,
        step_size: u16,
        max_bounces: u8,
        bg: Color,
        vfov: f32,
        look_from: P3,
        look_at: P3,
        v_up: V3,
        defocus_angle: f32,
        focus_dist: f32,
    ) -> Self {
        let image_height = max(1, (image_width as f32 / aspect_ratio) as u16);
        let center = look_from;

        let (iterations, samples_pp) = if step_size > 0 && samples_pp > step_size {
            (samples_pp / step_size, step_size)
        } else {
            (1, samples_pp)
        };

        // viewport dimensions
        let theta = vfov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h * focus_dist;
        let viewport_width = viewport_height * (image_width as f32 / image_height as f32);

        // Calculate the u,v,w unit basis vectors for the camera coordinate frame.
        let w = (look_from - look_at).normalize();
        let u = v_up.cross(w).normalize();
        let v = w.cross(u);

        let viewport_u = viewport_width * u;
        let viewport_v = viewport_height * -v;
        let pixel_delta_u = viewport_u / image_width as f32;
        let pixel_delta_v = viewport_v / image_height as f32;

        // Calculate the location of the upper left pixel.
        let viewport_upper_left = center - (focus_dist * w) - viewport_u / 2.0 - viewport_v / 2.0;
        let pixel_origin = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        // Calculate the camera defocus disk basis vectors.
        let defocus_radius = focus_dist * (defocus_angle / 2.0).to_radians().tan();
        let defocus_disk_u = u * defocus_radius;
        let defocus_disk_v = v * defocus_radius;

        Self {
            image_width,
            image_height,
            samples_pp,
            iterations,
            max_bounces,
            bg,
            center,
            pixel_origin,
            pixel_delta_u,
            pixel_delta_v,
            defocus_angle,
            defocus_disk_u,
            defocus_disk_v,
        }
    }

    pub fn dims(&self) -> (u32, u32) {
        (self.image_width as u32, self.image_height as u32)
    }

    pub fn render_sdl(&self, bvh: Bvh, mts: &mut MainThreadState, backend: &mut Backend<'_>) {
        let start = Instant::now();
        let w = self.image_width as usize;
        let h = self.image_height as usize;
        let mut pixels = vec![Color::default(); w * h];
        let mut new_pixels = vec![Color::default(); w * h];

        'iters: for i in 1..=self.iterations {
            let scale = 1.0 / (i * self.samples_pp) as f32;
            self.render_pass(&bvh, &mut new_pixels);

            let render_time = Instant::now().duration_since(start);
            eprintln!(
                "Render time so far ({i}/{}): {}s",
                self.iterations,
                render_time.as_secs()
            );

            new_pixels.par_iter_mut().for_each(|p| *p *= scale);

            if i == 1 {
                mem::swap(&mut pixels, &mut new_pixels);
            } else {
                let k = (i - 1) as f32 / i as f32;
                pixels
                    .iter_mut()
                    .zip(&new_pixels)
                    .for_each(|(px, new_px)| *px = *px * k + new_px);
            }

            backend.render(&pixels).unwrap();

            while let Some(evt) = mts.poll_event() {
                match evt {
                    Event::Quit { .. } => break 'iters,
                    Event::KeyDown {
                        keycode: Some(Keycode::Q | Keycode::Escape),
                        repeat: false,
                        ..
                    } => break 'iters,
                    _ => (),
                }
            }
        }

        eprintln!("writting ppm file");
        let s: String = pixels.iter().map(|c| color::ppm_string(*c)).collect();
        fs::write(
            "test.ppm",
            format!("P3\n{} {}\n255\n{s}", self.image_width, self.image_height),
        )
        .unwrap();

        let render_time = Instant::now().duration_since(start);
        eprintln!("\nRender time: {}s", render_time.as_secs());
    }

    pub fn render_pass(&self, bvh: &Bvh, pixels: &mut [Color]) {
        pixels
            .par_chunks_mut(self.image_width as usize)
            .enumerate()
            .for_each(|(j, row)| {
                let fj = j as f32;

                for (i, px) in row.iter_mut().enumerate() {
                    let fi = i as f32;
                    let mut acc = Color::default();

                    for _ in 0..self.samples_pp {
                        acc += self.ray_color(self.get_ray(fi, fj), bvh);
                    }

                    *px = acc;
                }
            });
    }

    /// Construct a camera ray originating from the defocus disk and directed at a randomly
    /// sampled point around the pixel location i, j.
    fn get_ray(&self, i: f32, j: f32) -> Ray {
        // Vector to a random point in the [-.5,-.5]-[+.5,+.5] unit square
        let offset = V3::new(random_range(-0.5..0.5), random_range(-0.5..0.5), 0.0);
        let sample = self.pixel_origin
            + ((i + offset.x) * self.pixel_delta_u)
            + ((j + offset.y) * self.pixel_delta_v);
        let ray_origin = if self.defocus_angle <= 0.0 {
            self.center
        } else {
            self.defocus_disk_sample()
        };

        Ray::new(ray_origin, sample - ray_origin)
    }

    // Returns a random point in the camera defocus disk.
    fn defocus_disk_sample(&self) -> P3 {
        let p = v3::random_in_unit_disk();

        self.center + (p.x * self.defocus_disk_u) + (p.y * self.defocus_disk_v)
    }

    fn ray_color(&self, mut r: Ray, bvh: &Bvh) -> Color {
        let mut incoming_light = color::BLACK;
        let mut rcolor = color::WHITE;
        let mut stack = [0; MAX_BVH_DEPTH];

        for _ in 0..self.max_bounces {
            let hr = match bvh.hits(&r, Interval::new(0.001, f32::INFINITY), &mut stack) {
                Some(hr) => hr,
                None => return rcolor * self.bg,
            };

            let emitted_light = hr.mat.color_emitted(hr.u, hr.v, hr.p);
            incoming_light += emitted_light * rcolor;

            match hr.mat.scatter(&r, &hr) {
                Some((scattered, attenuation)) => {
                    rcolor *= attenuation;
                    r = scattered;
                }
                None => break,
            };

            if (rcolor.x + rcolor.y + rcolor.z) < 0.0001 {
                break; // early exit if we can't contribute more light from here
            }
        }

        incoming_light
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub orig: P3,
    pub dir: V3,
    pub inv_dir: wide::f32x4,
    pub ro: wide::f32x4,
}

impl Ray {
    pub const fn new(orig: P3, dir: V3) -> Self {
        let ro = wide::f32x4::new([orig.x, orig.y, orig.z, 0.0]);
        let inv_dir = wide::f32x4::new([1.0 / dir.x, 1.0 / dir.y, 1.0 / dir.z, 0.0]);

        Self {
            orig,
            dir,
            inv_dir,
            ro,
        }
    }

    pub fn at(&self, t: f32) -> P3 {
        self.orig + t * self.dir
    }
}
