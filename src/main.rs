pub mod color;
pub mod hit;
pub mod material;
pub mod ray;
pub mod v3;

use std::io::stdout;

use color::Color;
use hit::{HitRecord, HittableList, Sphere};
use material::Material;
use rand::random_range;
use ray::{Camera, Ray};
use v3::{P3, V3};

// image aspect ratio
pub const ASPECT_RATIO: f64 = 16.0 / 10.0;
// image width in pixels
pub const IMAGE_WIDTH: u16 = 1400;
// vertical field of view in degrees
pub const VERTICAL_FOV: f64 = 20.0;
// number of random samples per pixel
pub const SAMPLES_PER_PIXEL: u16 = 500;
// maximum number of ray bounces allowed
pub const MAX_BOUNCES: u8 = 50;
// the point that the camera is looking from
pub const LOOK_FROM: P3 = P3::new(10.0, 3.0, 10.0);
// the point that the camera is looking to
pub const LOOK_AT: P3 = P3::new(0.0, 0.0, 0.0);
// the camera-relative "up" direction
pub const V_UP: V3 = V3::new(0.0, 1.0, 0.0);
// variation angle of rays through each pixel
pub const DEFOCUS_ANGLE: f64 = 0.05;
// distance from camera lookfrom point to plane of perfect focus
pub const FOCUS_DIST: f64 = 10.0;

fn main() {
    let mut world = HittableList::default();

    let m_ground = Material::lambertian(Color::new(0.5, 0.5, 0.5));
    world.add(Sphere::new(P3::new(0.0, -1000.0, -1.0), 1000.0, m_ground));

    for a in -11..11 {
        for b in -11..11 {
            let k = random_range(0.0..1.0);
            let center = P3::new(
                a as f64 + 0.9 * random_range(0.0..1.0),
                0.2,
                b as f64 + 0.9 * random_range(0.0..1.0),
            );

            if (center - P3::new(2.0, 0.2, 0.0)).length() > 0.9 {
                let mat = if k < 0.8 {
                    let albedo = Color::random(0.0, 1.0) * Color::random(0.0, 1.0);
                    Material::lambertian(albedo)
                } else if k < 0.9 {
                    let albedo = Color::random(0.5, 1.0);
                    let fuzz = random_range(0.1..0.5);
                    Material::metal(albedo, fuzz)
                } else {
                    Material::dielectric(1.5)
                };

                world.add(Sphere::new(center, 0.2, mat));
            }
        }
    }

    let m_center = Material::lambertian(Color::new(0.3, 0.2, 0.5));
    let m_left = Material::dielectric(1.5);
    let m_bubble = Material::dielectric(1.0 / 1.5);
    let m_right = Material::metal(Color::new(0.8, 0.6, 0.2), 0.02);

    world.add(Sphere::new(P3::new(0.0, 0.8, 0.0), 0.4, m_center));
    world.add(Sphere::new(P3::new(-1.0, 0.8, 0.0), 0.4, m_left));
    world.add(Sphere::new(P3::new(-1.0, 0.8, 0.0), 0.3, m_bubble));
    world.add(Sphere::new(P3::new(1.0, 0.8, 0.0), 0.4, m_right));

    let camera = Camera::new(
        ASPECT_RATIO,
        IMAGE_WIDTH,
        SAMPLES_PER_PIXEL,
        MAX_BOUNCES,
        VERTICAL_FOV,
        LOOK_FROM,
        LOOK_AT,
        V_UP,
        DEFOCUS_ANGLE,
        FOCUS_DIST,
    );
    eprintln!("{camera:#?}");
    eprintln!("Rendering...");
    camera.render_ppm(&mut stdout(), &world);
    eprintln!("\nDone");
}
