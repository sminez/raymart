pub mod bbox;
pub mod color;
pub mod hit;
pub mod material;
pub mod noise;
pub mod ray;
pub mod v3;

use rand::random_range;
use std::{env, io::stdout};

use bbox::BvhNode;
use color::Color;
use hit::{cuboid, HitRecord, Hittable, Quad, Sphere};
use material::Material;
use ray::{Camera, Ray};
use v3::{P3, V3};

pub const BG_COLOR: Color = Color::new(0.7, 0.8, 1.0); // default scene background color
pub const ASPECT_RATIO: f64 = 16.0 / 10.0; // image aspect ratio
pub const IMAGE_WIDTH: u16 = 1000; // image width in pixels
pub const SAMPLES_PER_PIXEL: u16 = 400; // number of random samples per pixel
pub const MAX_BOUNCES: u8 = 80; // maximum number of ray bounces allowed

macro_rules! p {
    ($x:expr, $y:expr, $z:expr) => {
        P3::new($x as f64, $y as f64, $z as f64)
    };
}

macro_rules! v {
    ($x:expr, $y:expr, $z:expr) => {
        V3::new($x as f64, $y as f64, $z as f64)
    };
}

fn main() {
    let sim = env::args().nth(1).unwrap_or_else(|| "default".to_string());
    eprintln!("sim = {sim}");

    let (hittables, camera) = match sim.as_str() {
        "random-ballscape" => random_ballscape(),
        "checkered-spheres" => checkered_spheres(),
        "composed" => composed(),
        "image" => image(),
        "perlin-spheres" => perlin_spheres(),
        "quads" => quads(),
        "simple-light" => simple_light(),
        "cornell" => empty_cornell_box(false),
        "mirror-cornell" => empty_cornell_box(true),
        "cornell-glass-ball" => cornell_box_glass_ball(),
        "mirror-cornell-glass-ball" => mirror_cornell_box_glass_ball(),
        "cornell-cuboids" => cornell_box_cuboids(),
        _ => cornell_box_cuboids(),
    };

    eprintln!("Computing bvh tree...");
    let bvh_tree = BvhNode::new_from_hittables(hittables);

    eprintln!("Rendering...");
    camera.render_ppm(&mut stdout(), &bvh_tree);

    eprintln!("\nDone");
}

pub fn image() -> (Vec<Hittable>, Camera) {
    let mut hittables = Vec::default();

    let m_ground = Material::checker(0.32, Color::new(0.5, 0.8, 0.2), Color::grey(0.9));
    hittables.push(Sphere::new(p!(0, -100.5, -1), 100.0, m_ground).into());

    let glass = Material::dielectric(1.33);
    let me = Material::image("sadfrog.png");

    hittables.push(Sphere::new(p!(0, 0, -1), 0.48, me).into());
    hittables.push(Sphere::new(p!(0, 0, -1), 0.50, glass).into());

    let vertical_fov = 10.0;
    let look_from = p!(3, 1, 11);
    let look_at = p!(0, 0, -1);
    let v_up = v!(0, 1, 0);
    let defocus_angle = 0.0;
    let focus_dist = 10.0;

    let camera = Camera::new(
        ASPECT_RATIO,
        IMAGE_WIDTH,
        SAMPLES_PER_PIXEL,
        MAX_BOUNCES,
        BG_COLOR,
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

    let m_ground = Material::checker(0.32, Color::new(0.5, 0.8, 0.2), Color::grey(0.9));
    hittables.push(Sphere::new(p!(0, -100.5, -1), 100.0, m_ground).into());

    let matte = Material::solid_color(Color::new(0.95, 0.15, 0.25));
    let glass = Material::dielectric(1.33);
    let air = Material::dielectric(1.0 / 1.33);
    let gold = Material::metal(Color::new(0.8, 0.6, 0.2), 0.02);
    let light = Material::diffuse_light(Color::grey(2.0));

    hittables.push(Sphere::new(p!(0, 0, -1), 0.48, matte).into());
    hittables.push(Sphere::new(p!(0, 0, -1), 0.50, glass).into());

    hittables.push(Sphere::new(p!(-1, 0, -1.2), 0.48, glass).into());
    hittables.push(Sphere::new(p!(-1, 0, -1.2), 0.45, air).into());
    hittables.push(Sphere::new(p!(-1, 0, -1.2), 0.42, glass).into());
    // hittables.push(Sphere::new(p!(-1, 0, -1.2), 0.39, gold).into());
    hittables.push(Sphere::new(p!(-1, 0, -1.2), 0.24, light).into());

    hittables.push(Sphere::new(p!(1, 0, -1), 0.48, gold).into());

    hittables.push(Sphere::new(p!(0.4, -0.31, 1), 0.22, glass).into());
    hittables.push(Sphere::new(p!(0.4, -0.31, 1), 0.1, light).into());

    hittables.push(Sphere::new(p!(-0.4, -0.3, 1), 0.2, gold).into());

    hittables.push(Sphere::new(p!(-0.7, -0.42, 1.2), 0.098, matte).into());
    hittables.push(Sphere::new(p!(-0.7, -0.42, 1.2), 0.1, glass).into());
    hittables.push(Sphere::new(p!(-0.1, -0.43, 1.6), 0.098, matte).into());
    hittables.push(Sphere::new(p!(-0.1, -0.43, 1.6), 0.1, glass).into());
    hittables.push(Sphere::new(p!(0.6, -0.44, 1.9), 0.098, matte).into());
    hittables.push(Sphere::new(p!(0.6, -0.44, 1.9), 0.1, glass).into());

    let vertical_fov = 10.0;
    let look_from = p!(0, 1, 11);
    let look_at = p!(0, 0, 0);
    let v_up = v!(0, 1, 0);
    let defocus_angle = 0.0;
    let focus_dist = 10.0;

    let camera = Camera::new(
        ASPECT_RATIO,
        IMAGE_WIDTH,
        SAMPLES_PER_PIXEL,
        MAX_BOUNCES,
        Color::grey(0.01),
        // BG_COLOR,
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

    let m_ground = Material::solid_color(Color::grey(0.5));
    hittables.push(Sphere::new(p!(0, -1000, -1), 1000.0, m_ground).into());

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
                    let center2 = center + v!(0, random_range(0.0..0.5), 0);

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

    hittables.push(Sphere::new(p!(0, 0.8, 0), 0.4, m_center).into());
    hittables.push(Sphere::new(p!(-1, 0.8, 0), 0.4, m_left).into());
    hittables.push(Sphere::new(p!(-1, 0.8, 0), 0.3, m_bubble).into());
    hittables.push(Sphere::new(p!(1, 0.8, 0), 0.4, m_right).into());

    let vertical_fov: f64 = 20.0;
    let look_from = p!(10, 3, 10);
    let look_at = P3::ORIGIN;
    let v_up = v!(0, 1, 0);
    let defocus_angle: f64 = 0.05;
    let focus_dist: f64 = 10.0;

    let camera = Camera::new(
        ASPECT_RATIO,
        IMAGE_WIDTH,
        SAMPLES_PER_PIXEL,
        MAX_BOUNCES,
        BG_COLOR,
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
    hittables.push(Sphere::new(p!(0, -10, 0), 10.0, m_checker).into());
    hittables.push(Sphere::new(p!(0, 10, 0), 10.0, m_checker).into());

    let vertical_fov: f64 = 20.0;
    let look_from = p!(12, 3, 3);
    let look_at = P3::ORIGIN;
    let v_up = v!(0, 1, 0);
    let defocus_angle: f64 = 0.05;
    let focus_dist: f64 = 10.0;

    let camera = Camera::new(
        ASPECT_RATIO,
        IMAGE_WIDTH,
        SAMPLES_PER_PIXEL,
        MAX_BOUNCES,
        BG_COLOR,
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

    let perlin = Material::noise(4.0);
    hittables.push(Sphere::new(p!(0, -1000, 0), 1000.0, perlin).into());
    hittables.push(Sphere::new(p!(0, 2, 0), 2.0, perlin).into());

    let vertical_fov: f64 = 20.0;
    let look_from = p!(13, 2, 3);
    let look_at = P3::ORIGIN;
    let v_up = v!(0, 1, 0);
    let defocus_angle: f64 = 0.05;
    let focus_dist: f64 = 10.0;

    let camera = Camera::new(
        ASPECT_RATIO,
        IMAGE_WIDTH,
        SAMPLES_PER_PIXEL,
        MAX_BOUNCES,
        BG_COLOR,
        vertical_fov,
        look_from,
        look_at,
        v_up,
        defocus_angle,
        focus_dist,
    );

    (hittables, camera)
}

pub fn quads() -> (Vec<Hittable>, Camera) {
    let mut hittables = Vec::default();

    let red = Material::solid_color(Color::new(1.0, 0.2, 0.2));
    let green = Material::solid_color(Color::new(0.2, 1.0, 0.2));
    let blue = Material::solid_color(Color::new(0.2, 0.2, 1.0));
    let orange = Material::solid_color(Color::new(1.0, 0.5, 0.0));
    let teal = Material::solid_color(Color::new(0.2, 0.8, 0.8));

    hittables.push(Quad::new_triangle(p!(-3, -2, 5), v!(0, 0, -4), v!(0, 4, 0), red).into());
    hittables.push(Quad::new_disk(p!(-2, -2, 0), v!(4, 0, 0), v!(0, 4, 0), 1.0, green).into());
    hittables.push(Quad::new_ring(p!(3, -2, 1), v!(0, 0, 4), v!(0, 4, 0), 1.0, 0.5, blue).into());
    hittables.push(Quad::new(p!(-2, 3, 1), v!(4, 0, 0), v!(0, 0, 4), orange).into());
    hittables.push(Quad::new(p!(-2, -3, 5), v!(4, 0, 0), v!(0, 0, -4), teal).into());

    let vertical_fov: f64 = 80.0;
    let look_from = p!(0, 0, 9);
    let look_at = P3::ORIGIN;
    let v_up = v!(0, 1, 0);
    let defocus_angle: f64 = 0.05;
    let focus_dist: f64 = 10.0;

    let camera = Camera::new(
        1.0,
        IMAGE_WIDTH,
        SAMPLES_PER_PIXEL,
        MAX_BOUNCES,
        BG_COLOR,
        vertical_fov,
        look_from,
        look_at,
        v_up,
        defocus_angle,
        focus_dist,
    );

    (hittables, camera)
}

pub fn simple_light() -> (Vec<Hittable>, Camera) {
    let mut hittables = Vec::default();

    let perlin = Material::noise(4.0);
    hittables.push(Sphere::new(p!(0, -1000, 0), 1000.0, perlin).into());
    hittables.push(Sphere::new(p!(0, 2, 0), 2.0, perlin).into());

    let light1 = Material::diffuse_light(Color::new(0.0, 2.0, 2.0));
    let light2 = Material::diffuse_light(Color::new(4.0, 0.0, 4.0));
    hittables.push(Sphere::new(p!(0, 8, 0), 2.0, light1).into());
    hittables.push(Quad::new(p!(3, 1, -2), v!(2, 0, 0), v!(0, 2, 0), light2).into());

    let vertical_fov: f64 = 20.0;
    let look_from = p!(26, 3, 6);
    let look_at = p!(0, 2, 0);
    let v_up = v!(0, 1, 0);
    let defocus_angle: f64 = 0.0;
    let focus_dist: f64 = 10.0;

    let camera = Camera::new(
        ASPECT_RATIO,
        IMAGE_WIDTH,
        SAMPLES_PER_PIXEL,
        MAX_BOUNCES,
        Color::BLACK,
        vertical_fov,
        look_from,
        look_at,
        v_up,
        defocus_angle,
        focus_dist,
    );

    (hittables, camera)
}

fn empty_cornell_box(white_mirror: bool) -> (Vec<Hittable>, Camera) {
    let mut hittables = Vec::default();

    // Ceiling light
    let light = Material::diffuse_light(Color::grey(25.0));
    hittables.push(Quad::new(p!(343, 554, 332), v!(-130, 0, 0), v!(0, 0, -105), light).into());

    // Walls
    let red = Material::solid_color(Color::new(0.65, 0.05, 0.05));
    let green = Material::solid_color(Color::new(0.12, 0.45, 0.15));
    let white = if white_mirror {
        Material::metal(Color::grey(0.93), 0.0)
    } else {
        Material::solid_color(Color::grey(0.73))
    };

    hittables.push(Quad::new(p!(555, 0, 0), v!(0, 555, 0), v!(0, 0, 555), green).into());
    hittables.push(Quad::new(p!(0, 0, 0), v!(0, 555, 0), v!(0, 0, 555), red).into());
    hittables.push(Quad::new(p!(0, 0, 0), v!(555, 0, 0), v!(0, 0, 555), white).into());
    hittables.push(Quad::new(p!(0, 0, 555), v!(555, 0, 0), v!(0, 555, 0), white).into());
    hittables.push(Quad::new(p!(555, 555, 555), v!(-555, 0, 0), v!(0, 0, -555), white).into());

    // Camera
    let vertical_fov = 40.0;
    let look_from = p!(278, 278, -800);
    let look_at = p!(278, 278, 0);
    let v_up = v!(0, 1, 0);
    let defocus_angle = 0.0;
    let focus_dist = 10.0;

    let camera = Camera::new(
        1.0,
        IMAGE_WIDTH,
        SAMPLES_PER_PIXEL,
        MAX_BOUNCES,
        Color::BLACK,
        vertical_fov,
        look_from,
        look_at,
        v_up,
        defocus_angle,
        focus_dist,
    );

    (hittables, camera)
}

pub fn cornell_box_glass_ball() -> (Vec<Hittable>, Camera) {
    let (mut hittables, camera) = empty_cornell_box(false);

    // Contents
    let air = Material::dielectric(1.0 / 1.33);
    let glass = Material::dielectric(1.33);

    hittables.push(Sphere::new(p!(343, 250, 342), 150.0, glass).into());
    hittables.push(Sphere::new(p!(343, 250, 342), 120.0, air).into());
    hittables.push(Sphere::new(p!(343, 250, 342), 100.0, glass).into());

    (hittables, camera)
}

pub fn mirror_cornell_box_glass_ball() -> (Vec<Hittable>, Camera) {
    let (mut hittables, camera) = empty_cornell_box(true);

    // Contents
    let air = Material::dielectric(1.0 / 1.33);
    let glass = Material::dielectric(1.33);

    hittables.push(Sphere::new(p!(343, 250, 342), 150.0, glass).into());
    hittables.push(Sphere::new(p!(343, 250, 342), 120.0, air).into());
    hittables.push(Sphere::new(p!(343, 250, 342), 100.0, glass).into());

    (hittables, camera)
}

pub fn cornell_box_cuboids() -> (Vec<Hittable>, Camera) {
    let (mut hittables, camera) = empty_cornell_box(false);

    // Contents
    let white = Material::solid_color(Color::grey(0.73));

    hittables.push(
        cuboid(p!(0, 0, 0), p!(165, 165, 165), white)
            .rotate(-18.0)
            .translate(v!(130, 0, 65)),
    );
    hittables.push(
        cuboid(p!(0, 0, 0), p!(165, 330, 165), white)
            .rotate(15.0)
            .translate(v!(265, 0, 295)),
    );

    (hittables, camera)
}

pub fn cornell_box_glass_cuboids() -> (Vec<Hittable>, Camera) {
    let (mut hittables, camera) = empty_cornell_box(false);

    // Contents
    let air = Material::dielectric(1.0 / 1.33);
    let glass = Material::dielectric(1.33);

    hittables.push(
        cuboid(p!(0, 0, 0), p!(165, 165, 165), glass)
            .rotate(-18.0)
            .translate(v!(130, 0, 65)),
    );
    hittables.push(
        cuboid(p!(0, 0, 0), p!(135, 135, 135), air)
            .rotate(-18.0)
            .translate(v!(130, 0, 65)),
    );
    hittables.push(
        cuboid(p!(0, 0, 0), p!(115, 115, 115), glass)
            .rotate(-18.0)
            .translate(v!(130, 0, 65)),
    );

    hittables.push(
        cuboid(p!(0, 0, 0), p!(165, 330, 165), glass)
            .rotate(15.0)
            .translate(v!(265, 0, 295)),
    );
    hittables.push(
        cuboid(p!(0, 0, 0), p!(135, 300, 135), air)
            .rotate(15.0)
            .translate(v!(265, 0, 295)),
    );
    hittables.push(
        cuboid(p!(0, 0, 0), p!(115, 290, 115), glass)
            .rotate(15.0)
            .translate(v!(265, 0, 295)),
    );

    (hittables, camera)
}
