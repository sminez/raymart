use crate::{
    hit::{Hittable, Interval},
    v3::{Point3, V3},
    Color,
};
use rayon::prelude::*;
use std::{cmp::max, io::Write};

pub const ASPECT_RATIO: f64 = 16.0 / 9.0;
pub const IMAGE_WIDTH: u16 = 800;
pub const FOCAL_LENGTH: f64 = 1.0;
pub const VIEWPORT_HEIGHT: f64 = 2.0;

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    image_width: u16,     // rendered image width (pixels)
    image_height: u16,    // rendered image height (pixels)
    center: Point3,       // camera center
    pixel_origin: Point3, // location of pixel 0,0
    pixel_delta_u: V3,    // offset to pixel to the right
    pixel_delta_v: V3,    // offset to pixel below
}

impl Camera {
    pub fn new(aspect_ratio: f64, image_width: u16) -> Self {
        let image_height = max(1, (image_width as f64 / aspect_ratio) as u16);
        let center = Point3::new(0.0, 0.0, 0.0);

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
            center,
            pixel_origin,
            pixel_delta_u,
            pixel_delta_v,
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
                    let pixel_center = self.pixel_origin
                        + (i as f64 * self.pixel_delta_u)
                        + (j as f64 * self.pixel_delta_v);
                    let r = Ray::new(self.center, pixel_center - self.center);

                    self.ray_color(&r, world).ppm_string()
                })
            })
            .collect();

        writeln!(w, "{}", lines.join("\n")).unwrap();
    }

    fn ray_color(&self, r: &Ray, world: &impl Hittable) -> Color {
        if let Some(hit_record) = world.hits(r, Interval::new(0.0, f64::INFINITY)) {
            return 0.5 * (hit_record.normal + Color::new(1.0, 1.0, 1.0));
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
