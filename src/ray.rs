use crate::{
    hit::{Hittable, Interval},
    v3::{Point3, V3},
    Color, FOCAL_LENGTH, VIEWPORT_HEIGHT,
};
use rand::random_range;
use rayon::prelude::*;
use std::{cmp::max, io::Write};

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    image_width: u16,        // rendered image width (pixels)
    image_height: u16,       // rendered image height (pixels)
    samples_pp: u8,          // number of random samples per pixel
    max_bounces: u8,         // maximum number of ray bounces allowed
    center: Point3,          // camera center
    pixel_origin: Point3,    // location of pixel 0,0
    pixel_delta_u: V3,       // offset to pixel to the right
    pixel_delta_v: V3,       // offset to pixel below
    pixel_sample_scale: f64, // color scale factor for a sum of pixel samples
}

impl Camera {
    pub fn new(aspect_ratio: f64, image_width: u16, samples_pp: u8, max_bounces: u8) -> Self {
        let image_height = max(1, (image_width as f64 / aspect_ratio) as u16);
        let center = Point3::new(0.0, 0.0, 0.0);
        let pixel_sample_scale = 1.0 / samples_pp as f64;

        // viewport dimensions
        let viewport_width = VIEWPORT_HEIGHT * (image_width as f64 / image_height as f64);
        let viewport_u = V3::new(viewport_width, 0.0, 0.0);
        let viewport_v = V3::new(0.0, -VIEWPORT_HEIGHT, 0.0);
        let pixel_delta_u = viewport_u / image_width as f64;
        let pixel_delta_v = viewport_v / image_height as f64;

        // Calculate the location of the upper left pixel.
        let viewport_upper_left =
            center - V3::new(0.0, 0.0, FOCAL_LENGTH) - viewport_u / 2.0 - viewport_v / 2.0;
        let pixel_origin = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        Self {
            image_width,
            image_height,
            samples_pp,
            max_bounces,
            center,
            pixel_origin,
            pixel_delta_u,
            pixel_delta_v,
            pixel_sample_scale,
        }
    }

    pub fn render_ppm(&self, w: &mut impl Write, world: &impl Hittable) {
        if let Err(e) = writeln!(w, "P3\n{} {}\n255", self.image_width, self.image_height) {
            panic!("unable to write ppm header: {e}");
        }

        let lines: Vec<String> = (0..self.image_height)
            .into_par_iter()
            .flat_map(move |j| {
                (0..self.image_width).into_par_iter().map(move |i| {
                    let color = (0..self.samples_pp)
                        .into_par_iter()
                        .map(|_| {
                            let r = self.get_ray(i, j);
                            self.ray_color(&r, self.max_bounces, world)
                        })
                        .reduce(Color::default, |a, b| a + b);

                    (color * self.pixel_sample_scale).ppm_string()
                })
            })
            .collect();

        writeln!(w, "{}", lines.join("\n")).unwrap();
    }

    /// Construct a camera ray originating from the origin and directed at randomly sampled
    /// point around the pixel location i, j.
    fn get_ray(&self, i: u16, j: u16) -> Ray {
        // Vector to a random point in the [-.5,-.5]-[+.5,+.5] unit square
        let offset = V3::new(random_range(-0.5..0.5), random_range(-0.5..0.5), 0.0);
        let sample = self.pixel_origin
            + ((i as f64 + offset.x) * self.pixel_delta_u)
            + ((j as f64 + offset.y) * self.pixel_delta_v);

        Ray::new(self.center, sample - self.center)
    }

    fn ray_color(&self, r: &Ray, depth: u8, world: &impl Hittable) -> Color {
        if depth == 0 {
            return Color::default();
        }

        if let Some(hit_record) = world.hits(r, Interval::new(0.0, f64::INFINITY)) {
            let v = V3::random_on_hemisphere(&hit_record.normal);
            return 0.5 * self.ray_color(&Ray::new(hit_record.p, v), depth - 1, world);
        }

        let unit_dir = r.dir.unit_vector();
        let a = 0.5 * (unit_dir.y + 1.0);

        (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
    }
}

pub struct Ray {
    pub orig: Point3,
    pub dir: V3,
}

impl Ray {
    pub const fn new(orig: Point3, dir: V3) -> Self {
        Self { orig, dir }
    }

    pub fn at(&self, t: f64) -> Point3 {
        self.orig + t * self.dir
    }
}
