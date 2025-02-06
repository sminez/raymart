use crate::{
    bbox::BvhNode,
    hit::Interval,
    v3::{P3, V3},
    Color,
};
use rand::random_range;
use rayon::prelude::*;
use std::{cmp::max, io::Write, time::Instant};

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    image_width: u16,        // rendered image width (pixels)
    image_height: u16,       // rendered image height (pixels)
    samples_pp: u16,         // number of random samples per pixel
    max_bounces: u8,         // maximum number of ray bounces allowed
    bg: Color,               // scene background color
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
        bg: Color,
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
            bg,
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

    pub fn set_samples_per_pixel(&mut self, samples_pp: u16) {
        self.samples_pp = samples_pp;
        self.pixel_sample_scale = 1.0 / samples_pp as f64;
    }

    pub fn render_ppm(&self, w: &mut impl Write, world: &BvhNode) {
        if let Err(e) = writeln!(w, "P3\n{} {}\n255", self.image_width, self.image_height) {
            panic!("unable to write ppm header: {e}");
        }

        let start = Instant::now();
        let pixels: Vec<Color> = (0..self.image_height)
            .into_par_iter()
            .flat_map(move |j| {
                let res = (0..self.image_width).into_par_iter().map(move |i| {
                    let (fi, fj) = (i as f64, j as f64);
                    let color = (0..self.samples_pp)
                        .into_par_iter()
                        .map(|_| self.ray_color(self.get_ray(fi, fj), world))
                        .reduce(Color::default, |mut a, b| {
                            a += b;
                            a
                        });

                    color * self.pixel_sample_scale
                });
                eprint!(".");
                res
            })
            .collect();

        let s: String = pixels.into_iter().map(|c| c.ppm_string()).collect();
        writeln!(w, "{s}").unwrap();

        let render_time = Instant::now().duration_since(start);
        eprintln!("\nRender time: {}s", render_time.as_secs());
    }

    /// Construct a camera ray originating from the defocus disk and directed at a randomly
    /// sampled point around the pixel location i, j.
    fn get_ray(&self, i: f64, j: f64) -> Ray {
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

        Ray::new(self.center, sample - ray_origin)
    }

    // Returns a random point in the camera defocus disk.
    fn defocus_disk_sample(&self) -> P3 {
        let p = V3::random_in_unit_disk();

        self.center + (p.x * self.defocus_disk_u) + (p.y * self.defocus_disk_v)
    }

    fn ray_color(&self, mut r: Ray, world: &BvhNode) -> Color {
        let mut incoming_light = Color::BLACK;
        let mut rcolor = Color::WHITE;

        for _ in 0..self.max_bounces {
            let hr = match world.hits(&r, Interval::new(0.001, f64::INFINITY)) {
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
    pub inv_dir: wide::f64x4,
    pub ro: wide::f64x4,
}

impl Ray {
    pub const fn new(orig: P3, dir: V3) -> Self {
        let ro = wide::f64x4::new([orig.x, orig.y, orig.z, 0.0]);
        let inv_dir = wide::f64x4::new([1.0 / dir.x, 1.0 / dir.y, 1.0 / dir.z, 0.0]);

        Self {
            orig,
            dir,
            inv_dir,
            ro,
        }
    }

    pub fn at(&self, t: f64) -> P3 {
        self.orig + t * self.dir
    }
}
