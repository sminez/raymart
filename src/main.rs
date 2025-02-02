pub mod color;
pub mod hit;
pub mod ray;
pub mod v3;

use std::io::stdout;

use color::Color;
use hit::{HittableList, Sphere};
use ray::{Camera, Ray};
use v3::{Point3, V3};

pub const ASPECT_RATIO: f64 = 16.0 / 9.0;
pub const IMAGE_WIDTH: u16 = 800;
pub const FOCAL_LENGTH: f64 = 1.0;
pub const VIEWPORT_HEIGHT: f64 = 2.0;
pub const SAMPLES_PER_PIXEL: u8 = 100;

fn main() {
    let mut world = HittableList::default();
    world.add(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5));
    world.add(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0));

    let camera = Camera::new(ASPECT_RATIO, IMAGE_WIDTH, SAMPLES_PER_PIXEL);
    eprintln!("{camera:#?}");
    eprintln!("Rendering...");
    camera.render_ppm(&mut stdout(), &world);
    eprintln!("Done");
}
