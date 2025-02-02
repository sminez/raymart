pub mod color;
pub mod hit;
pub mod ray;
pub mod v3;

use color::Color;
use hit::{HittableList, Sphere};
use ray::Ray;
use v3::{Point3, V3};

const ASPECT_RATIO: f64 = 16.0 / 9.0;
const IMAGE_WIDTH: u16 = 800;
const IMAGE_HEIGHT: u16 = calculate_image_height();

const FOCAL_LENGTH: f64 = 1.0;
const VIEWPORT_HEIGHT: f64 = 2.0;
const VIEWPORT_WIDTH: f64 = VIEWPORT_HEIGHT * (IMAGE_WIDTH as f64 / IMAGE_HEIGHT as f64);

// Calculate the image height, and ensure that it's at least 1.
const fn calculate_image_height() -> u16 {
    let h = IMAGE_WIDTH as f64 / ASPECT_RATIO;
    if h < 1.0 {
        1
    } else {
        h as u16
    }
}

fn main() {
    // World
    let mut world = HittableList::default();
    world.add(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5));
    world.add(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0));

    // Camera
    let camera_center = Point3::new(0.0, 0.0, 0.0);

    // Calculate the vectors across the horizontal and down the vertical viewport edges.
    let viewport_u = V3::new(VIEWPORT_WIDTH, 0.0, 0.0);
    let viewport_v = V3::new(0.0, -VIEWPORT_HEIGHT, 0.0);

    // Calculate the horizontal and vertical delta vectors from pixel to pixel.
    let pixel_delta_u = viewport_u / IMAGE_WIDTH as f64;
    let pixel_delta_v = viewport_v / IMAGE_HEIGHT as f64;

    // Calculate the location of the upper left pixel.
    let viewport_upper_left =
        camera_center - V3::new(0.0, 0.0, FOCAL_LENGTH) - viewport_u / 2.0 - viewport_v / 2.0;
    let pixel_100_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

    eprintln!("image height: {IMAGE_HEIGHT}");
    eprintln!("viewport width: {VIEWPORT_WIDTH}");
    eprintln!("viewport u: {viewport_u:?}");
    eprintln!("viewport v: {viewport_v:?}");
    eprintln!("pixel delta u: {pixel_delta_u:?}");
    eprintln!("pixel delta v: {pixel_delta_v:?}");
    eprintln!("viewport upper left: {viewport_upper_left:?}");
    eprintln!("pixel 100 loc: {pixel_100_loc:?}");

    // render
    println!("P3\n{IMAGE_WIDTH} {IMAGE_HEIGHT}\n255");

    for j in 0..IMAGE_HEIGHT {
        // eprintln!("Scanlines remaining: {}", IMAGE_HEIGHT - j);
        eprint!(".");
        for i in 0..IMAGE_WIDTH {
            let pixel_center =
                pixel_100_loc + (i as f64 * pixel_delta_u) + (j as f64 * pixel_delta_v);
            let ray_direction = pixel_center - camera_center;
            let ray = Ray::new(camera_center, ray_direction);
            ray.color(&world).print_ppm();
        }
    }

    eprintln!("\nDone");
}
