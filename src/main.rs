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

pub const ASPECT_RATIO: f64 = 16.0 / 10.0; // image aspect ratio
pub const IMAGE_WIDTH: u16 = 1600; // image width in pixels
pub const SAMPLES_PER_PIXEL: u16 = 10; // number of random samples per pixel
pub const MAX_BOUNCES: u8 = 5; // maximum number of ray bounces allowed

fn main() {
    let (world, camera) = random_ballscape();
    // let (world, camera) = composed();

    eprintln!("{camera:#?}");
    eprintln!("Rendering...");
    camera.render_ppm(&mut stdout(), &world);

    eprintln!("\nDone");
}

pub fn composed() -> (HittableList, Camera) {
    let mut world = HittableList::default();

    let m_ground = Material::lambertian(Color::new(0.5, 0.8, 0.2));
    world.add(Sphere::new(P3::new(0.0, -100.5, -1.0), 100.0, m_ground));

    let matte = Material::lambertian(Color::new(0.95, 0.15, 0.25));
    let glass = Material::dielectric(1.33);
    let air = Material::dielectric(1.0 / 1.33);
    let gold = Material::metal(Color::new(0.8, 0.6, 0.2), 0.02);

    world.add(Sphere::new(P3::new(0.0, 0.0, -1.0), 0.48, matte));

    world.add(Sphere::new(P3::new(-1.0, 0.0, -1.2), 0.48, glass));
    world.add(Sphere::new(P3::new(-1.0, 0.0, -1.2), 0.45, air));
    world.add(Sphere::new(P3::new(-1.0, 0.0, -1.2), 0.42, glass));
    world.add(Sphere::new(P3::new(-1.0, 0.0, -1.2), 0.39, gold));

    world.add(Sphere::new(P3::new(1.0, 0.0, -1.0), 0.48, gold));

    world.add(Sphere::new(P3::new(0.4, -0.31, 1.0), 0.22, glass));
    world.add(Sphere::new(P3::new(-0.4, -0.3, 1.0), 0.2, gold));

    world.add(Sphere::new(P3::new(-0.7, -0.42, 1.2), 0.1, matte));
    world.add(Sphere::new(P3::new(-0.1, -0.43, 1.6), 0.1, matte));
    world.add(Sphere::new(P3::new(0.6, -0.44, 1.9), 0.1, matte));

    // vertical field of view in degrees
    let vertical_fov: f64 = 10.0;
    // the point that the camera is looking from
    let look_from: P3 = P3::new(0.0, 1.0, 11.0);
    // the point that the camera is looking to
    let look_at: P3 = P3::new(0.0, 0.0, 0.0);
    // the camera-relative "up" direction
    let v_up: V3 = V3::new(0.0, 1.0, 0.0);
    // variation angle of rays through each pixel
    let defocus_angle: f64 = 0.0;
    // distance from camera lookfrom point to plane of perfect focus
    let focus_dist: f64 = 10.0;

    let camera = Camera::new(
        ASPECT_RATIO,
        IMAGE_WIDTH,
        SAMPLES_PER_PIXEL,
        MAX_BOUNCES,
        vertical_fov,
        look_from,
        look_at,
        v_up,
        defocus_angle,
        focus_dist,
    );

    (world, camera)
}

/// The final image from "Ray tracing in one weekend"
pub fn random_ballscape() -> (HittableList, Camera) {
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
                    let mat = Material::lambertian(albedo);
                    let center2 = center + V3::new(0.0, random_range(0.0..0.5), 0.0);

                    world.add(Sphere::new_moving(center, center2, 0.2, mat));
                    continue;
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

    // vertical field of view in degrees
    let vertical_fov: f64 = 20.0;
    // the point that the camera is looking from
    let look_from: P3 = P3::new(10.0, 3.0, 10.0);
    // the point that the camera is looking to
    let look_at: P3 = P3::new(0.0, 0.0, 0.0);
    // the camera-relative "up" direction
    let v_up: V3 = V3::new(0.0, 1.0, 0.0);
    // variation angle of rays through each pixel
    let defocus_angle: f64 = 0.05;
    // distance from camera lookfrom point to plane of perfect focus
    let focus_dist: f64 = 10.0;

    let camera = Camera::new(
        ASPECT_RATIO,
        IMAGE_WIDTH,
        SAMPLES_PER_PIXEL,
        MAX_BOUNCES,
        vertical_fov,
        look_from,
        look_at,
        v_up,
        defocus_angle,
        focus_dist,
    );

    (world, camera)
}
