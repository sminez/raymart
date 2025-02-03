pub mod bbox;
pub mod color;
pub mod hit;
pub mod material;
pub mod noise;
pub mod ray;
pub mod v3;

use rand::random_range;
use std::io::stdout;

use bbox::BvhNode;
use color::Color;
use hit::{HitRecord, Hittable, Sphere};
use material::Material;
use ray::{Camera, Ray};
use v3::{P3, V3};

pub const ASPECT_RATIO: f64 = 16.0 / 10.0; // image aspect ratio
pub const IMAGE_WIDTH: u16 = 1600; // image width in pixels
pub const SAMPLES_PER_PIXEL: u16 = 100; // number of random samples per pixel
pub const MAX_BOUNCES: u8 = 50; // maximum number of ray bounces allowed

fn main() {
    // let (hittables, camera) = random_ballscape();
    // let (hittables, camera) = checkered_spheres();
    // let (hittables, camera) = composed();
    // let (hittables, camera) = image();
    let (hittables, camera) = perlin_spheres();

    eprintln!("Computing bvh tree...");
    // There is definitely a break even point in terms of the number of number of hittables
    // in the scene and the utility of the bvh_tree in terms of the overhead from checking
    // hits against the bounding boxes.
    // It's probably worth defining a heuristic to check against the resulting tree to see
    // if it is worthwhile using it or not.
    let bvh_tree = BvhNode::new_from_hittables(hittables);

    eprintln!("Rendering...");
    camera.render_ppm(&mut stdout(), &bvh_tree);

    eprintln!("\nDone");
}

pub fn image() -> (Vec<Hittable>, Camera) {
    let mut hittables = Vec::default();

    let m_ground = Material::checker(0.32, Color::new(0.5, 0.8, 0.2), Color::new(0.9, 0.9, 0.9));
    hittables.push(Sphere::new(P3::new(0.0, -100.5, -1.0), 100.0, m_ground).into());

    let glass = Material::dielectric(1.33);
    let me = Material::image("sadfrog.png");

    hittables.push(Sphere::new(P3::new(0.0, 0.0, -1.0), 0.48, me).into());
    hittables.push(Sphere::new(P3::new(0.0, 0.0, -1.0), 0.50, glass).into());

    let vertical_fov: f64 = 10.0;
    let look_from: P3 = P3::new(3.0, 1.0, 11.0);
    let look_at: P3 = P3::new(0.0, 0.0, -1.0);
    let v_up: V3 = V3::new(0.0, 1.0, 0.0);
    let defocus_angle: f64 = 0.0;
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

    (hittables, camera)
}

pub fn composed() -> (Vec<Hittable>, Camera) {
    let mut hittables = Vec::default();

    let m_ground = Material::checker(0.32, Color::new(0.5, 0.8, 0.2), Color::new(0.9, 0.9, 0.9));
    hittables.push(Sphere::new(P3::new(0.0, -100.5, -1.0), 100.0, m_ground).into());

    let matte = Material::solid_color(Color::new(0.95, 0.15, 0.25));
    let glass = Material::dielectric(1.33);
    let air = Material::dielectric(1.0 / 1.33);
    let gold = Material::metal(Color::new(0.8, 0.6, 0.2), 0.02);

    hittables.push(Sphere::new(P3::new(0.0, 0.0, -1.0), 0.48, matte.clone()).into());
    hittables.push(Sphere::new(P3::new(0.0, 0.0, -1.0), 0.50, glass.clone()).into());

    hittables.push(Sphere::new(P3::new(-1.0, 0.0, -1.2), 0.48, glass.clone()).into());
    hittables.push(Sphere::new(P3::new(-1.0, 0.0, -1.2), 0.45, air.clone()).into());
    hittables.push(Sphere::new(P3::new(-1.0, 0.0, -1.2), 0.42, glass.clone()).into());
    hittables.push(Sphere::new(P3::new(-1.0, 0.0, -1.2), 0.39, gold.clone()).into());

    hittables.push(Sphere::new(P3::new(1.0, 0.0, -1.0), 0.48, gold.clone()).into());

    hittables.push(Sphere::new(P3::new(0.4, -0.31, 1.0), 0.22, glass.clone()).into());
    hittables.push(Sphere::new(P3::new(-0.4, -0.3, 1.0), 0.2, gold.clone()).into());

    hittables.push(Sphere::new(P3::new(-0.7, -0.42, 1.2), 0.098, matte.clone()).into());
    hittables.push(Sphere::new(P3::new(-0.7, -0.42, 1.2), 0.1, glass.clone()).into());
    hittables.push(Sphere::new(P3::new(-0.1, -0.43, 1.6), 0.098, matte.clone()).into());
    hittables.push(Sphere::new(P3::new(-0.1, -0.43, 1.6), 0.1, glass.clone()).into());
    hittables.push(Sphere::new(P3::new(0.6, -0.44, 1.9), 0.098, matte.clone()).into());
    hittables.push(Sphere::new(P3::new(0.6, -0.44, 1.9), 0.1, glass.clone()).into());

    let vertical_fov: f64 = 10.0;
    let look_from: P3 = P3::new(0.0, 1.0, 11.0);
    let look_at: P3 = P3::new(0.0, 0.0, 0.0);
    let v_up: V3 = V3::new(0.0, 1.0, 0.0);
    let defocus_angle: f64 = 0.0;
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

    (hittables, camera)
}

/// The final image from "Ray tracing in one weekend"
pub fn random_ballscape() -> (Vec<Hittable>, Camera) {
    let mut hittables = Vec::default();

    let m_ground = Material::solid_color(Color::new(0.5, 0.5, 0.5));
    hittables.push(Sphere::new(P3::new(0.0, -1000.0, -1.0), 1000.0, m_ground).into());

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
                    let mat = Material::solid_color(albedo);
                    let center2 = center + V3::new(0.0, random_range(0.0..0.5), 0.0);

                    hittables.push(Sphere::new_moving(center, center2, 0.2, mat).into());
                    continue;
                } else if k < 0.9 {
                    let albedo = Color::random(0.5, 1.0);
                    let fuzz = random_range(0.1..0.5);
                    Material::metal(albedo, fuzz)
                } else {
                    Material::dielectric(1.5)
                };

                hittables.push(Sphere::new(center, 0.2, mat).into());
            }
        }
    }

    let m_center = Material::solid_color(Color::new(0.3, 0.2, 0.5));
    let m_left = Material::dielectric(1.5);
    let m_bubble = Material::dielectric(1.0 / 1.5);
    let m_right = Material::metal(Color::new(0.8, 0.6, 0.2), 0.02);

    hittables.push(Sphere::new(P3::new(0.0, 0.8, 0.0), 0.4, m_center).into());
    hittables.push(Sphere::new(P3::new(-1.0, 0.8, 0.0), 0.4, m_left).into());
    hittables.push(Sphere::new(P3::new(-1.0, 0.8, 0.0), 0.3, m_bubble).into());
    hittables.push(Sphere::new(P3::new(1.0, 0.8, 0.0), 0.4, m_right).into());

    let vertical_fov: f64 = 20.0;
    let look_from: P3 = P3::new(10.0, 3.0, 10.0);
    let look_at: P3 = P3::new(0.0, 0.0, 0.0);
    let v_up: V3 = V3::new(0.0, 1.0, 0.0);
    let defocus_angle: f64 = 0.05;
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

    (hittables, camera)
}

pub fn checkered_spheres() -> (Vec<Hittable>, Camera) {
    let mut hittables = Vec::default();

    let m_checker = Material::checker(0.32, Color::new(0.2, 0.3, 0.1), Color::new(0.9, 0.9, 0.9));
    hittables.push(Sphere::new(P3::new(0.0, -10.0, 0.0), 10.0, m_checker.clone()).into());
    hittables.push(Sphere::new(P3::new(0.0, 10.0, 0.0), 10.0, m_checker).into());

    let vertical_fov: f64 = 20.0;
    let look_from: P3 = P3::new(12.0, 3.0, 3.0);
    let look_at: P3 = P3::new(0.0, 0.0, 0.0);
    let v_up: V3 = V3::new(0.0, 1.0, 0.0);
    let defocus_angle: f64 = 0.05;
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

    (hittables, camera)
}

pub fn perlin_spheres() -> (Vec<Hittable>, Camera) {
    let mut hittables = Vec::default();

    let perlin = Material::noise();
    hittables.push(Sphere::new(P3::new(0.0, -1000.0, 0.0), 1000.0, perlin.clone()).into());
    hittables.push(Sphere::new(P3::new(0.0, 2.0, 0.0), 2.0, perlin).into());

    let vertical_fov: f64 = 20.0;
    let look_from: P3 = P3::new(13.0, 2.0, 3.0);
    let look_at: P3 = P3::new(0.0, 0.0, 0.0);
    let v_up: V3 = V3::new(0.0, 1.0, 0.0);
    let defocus_angle: f64 = 0.05;
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

    (hittables, camera)
}
