use crate::{
    hit::{Hittable, Interval},
    v3::{P3, V3},
    Color,
};
use rand::random_range;
use rayon::prelude::*;
use std::{cmp::max, io::Write};

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    image_width: u16,        // rendered image width (pixels)
    image_height: u16,       // rendered image height (pixels)
    samples_pp: u16,         // number of random samples per pixel
    max_bounces: u8,         // maximum number of ray bounces allowed
    center: P3,              // camera center
    pixel_origin: P3,        // location of pixel 0,0
    pixel_delta_u: V3,       // offset to pixel to the right
    pixel_delta_v: V3,       // offset to pixel below
    pixel_sample_scale: f64, // color scale factor for a sum of pixel samples
    defocus_angle: f64,
    defocus_disk_u: V3, // defocus disk horizontal radius
    defocus_disk_v: V3, // defocus disk vertical radius
}

impl Camera {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        aspect_ratio: f64,
        image_width: u16,
        samples_pp: u16,
        max_bounces: u8,
        vfov: f64,
        look_from: P3,
        look_at: P3,
        v_up: V3,
        defocus_angle: f64,
        focus_dist: f64,
    ) -> Self {
        let image_height = max(1, (image_width as f64 / aspect_ratio) as u16);
        let center = look_from;
        let pixel_sample_scale = 1.0 / samples_pp as f64;

        // viewport dimensions
        let theta = vfov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h * focus_dist;
        let viewport_width = viewport_height * (image_width as f64 / image_height as f64);

        // Calculate the u,v,w unit basis vectors for the camera coordinate frame.
        let w = (look_from - look_at).unit_vector();
        let u = v_up.cross(&w);
        let v = w.cross(&u);

        let viewport_u = viewport_width * u;
        let viewport_v = viewport_height * -v;
        let pixel_delta_u = viewport_u / image_width as f64;
        let pixel_delta_v = viewport_v / image_height as f64;

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
            max_bounces,
            center,
            pixel_origin,
            pixel_delta_u,
            pixel_delta_v,
            pixel_sample_scale,
            defocus_angle,
            defocus_disk_u,
            defocus_disk_v,
        }
    }

    pub fn render_ppm(&self, w: &mut impl Write, world: &impl Hittable) {
        if let Err(e) = writeln!(w, "P3\n{} {}\n255", self.image_width, self.image_height) {
            panic!("unable to write ppm header: {e}");
        }

        let lines: Vec<String> = (0..self.image_height)
            .into_par_iter()
            .flat_map(move |j| {
                let res = (0..self.image_width).into_par_iter().map(move |i| {
                    let color = (0..self.samples_pp)
                        .into_par_iter()
                        .map(|_| self.get_ray(i, j).color(self.max_bounces, world))
                        .reduce(Color::default, |a, b| a + b);

                    (color * self.pixel_sample_scale).ppm_string()
                });
                eprint!(".");
                res
            })
            .collect();

        writeln!(w, "{}", lines.join("\n")).unwrap();
    }

    /// Construct a camera ray originating from the defocus disk and directed at a randomly
    /// sampled point around the pixel location i, j.
    fn get_ray(&self, i: u16, j: u16) -> Ray {
        // Vector to a random point in the [-.5,-.5]-[+.5,+.5] unit square
        let offset = V3::new(random_range(-0.5..0.5), random_range(-0.5..0.5), 0.0);
        let sample = self.pixel_origin
            + ((i as f64 + offset.x) * self.pixel_delta_u)
            + ((j as f64 + offset.y) * self.pixel_delta_v);
        let ray_origin = if self.defocus_angle <= 0.0 {
            self.center
        } else {
            self.defocus_disk_sample()
        };

        Ray::new(self.center, sample - ray_origin)
    }

    // Returns a random point in the camera defocus disk.
    fn defocus_disk_sample(&self) -> P3 {
        let p = V3::random_in_unit_disk();

        self.center + (p.x * self.defocus_disk_u) + (p.y * self.defocus_disk_v)
    }
}

pub struct Ray {
    pub orig: P3,
    pub dir: V3,
}

impl Ray {
    pub const fn new(orig: P3, dir: V3) -> Self {
        Self { orig, dir }
    }

    pub fn at(&self, t: f64) -> P3 {
        self.orig + t * self.dir
    }

    fn color(&self, depth: u8, world: &impl Hittable) -> Color {
        if depth == 0 {
            return Color::default();
        }

        if let Some(hit_record) = world.hits(self, Interval::new(0.001, f64::INFINITY)) {
            return match hit_record.mat.scatter(self, &hit_record) {
                Some((scattered, attenuation)) => attenuation * scattered.color(depth - 1, world),
                None => Color::default(),
            };
        }

        let unit_dir = self.dir.unit_vector();
        let a = 0.5 * (unit_dir.y + 1.0);

        (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
    }
}
