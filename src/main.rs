pub mod color;
pub mod hit;
pub mod ray;
pub mod v3;

use std::io::stdout;

use color::Color;
use hit::{HittableList, Sphere};
use ray::{Camera, Ray, ASPECT_RATIO, IMAGE_WIDTH};
use v3::{Point3, V3};

fn main() {
    let mut world = HittableList::default();
    world.add(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5));
    world.add(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0));

    let camera = Camera::new(ASPECT_RATIO, IMAGE_WIDTH);
    eprintln!("{camera:#?}");
    eprintln!("Rendering...");
    camera.render_ppm(&mut stdout(), &world);
    eprintln!("Done");
}
