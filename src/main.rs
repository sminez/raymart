pub mod color;
pub mod hit;
pub mod material;
pub mod ray;
pub mod v3;

use std::io::stdout;

use color::Color;
use hit::{HitRecord, HittableList, Sphere};
use material::Material;
use ray::{Camera, Ray};
use v3::{Point3, V3};

pub const ASPECT_RATIO: f64 = 16.0 / 9.0;
pub const IMAGE_WIDTH: u16 = 800;
pub const FOCAL_LENGTH: f64 = 1.0;
pub const VERTICAL_FOV: f64 = 50.0;
pub const SAMPLES_PER_PIXEL: u8 = 100;
pub const MAX_BOUNCES: u8 = 50;

fn main() {
    // Materials
    let m_ground = Material::lambertian(Color::new(0.8, 0.8, 0.0));
    let m_center = Material::lambertian(Color::new(0.1, 0.2, 0.5));
    let m_left = Material::dielectric(1.5);
    let m_bubble = Material::dielectric(1.0 / 1.5);
    let m_right = Material::metal(Color::new(0.8, 0.6, 0.2), 0.03);

    let mut world = HittableList::default();
    world.add(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0, m_ground));
    world.add(Sphere::new(Point3::new(0.0, 0.0, -1.5), 0.5, m_center));
    world.add(Sphere::new(Point3::new(-1.0, 0.0, -1.1), 0.5, m_left));
    world.add(Sphere::new(Point3::new(-1.0, 0.0, -1.1), 0.4, m_bubble));
    world.add(Sphere::new(Point3::new(1.0, 0.0, -1.1), 0.5, m_right));

    let camera = Camera::new(
        ASPECT_RATIO,
        IMAGE_WIDTH,
        SAMPLES_PER_PIXEL,
        MAX_BOUNCES,
        VERTICAL_FOV,
    );
    eprintln!("{camera:#?}");
    eprintln!("Rendering...");
    camera.render_ppm(&mut stdout(), &world);
    eprintln!("Done");
}
