pub mod color;
pub mod v3;

use color::Color;

fn main() {
    output_ppm(256, 256);
}

fn output_ppm(w: u16, h: u16) {
    println!("P3\n{w} {h}\n255");

    for j in 0..h {
        eprintln!("Scanlines remaining: {}", h - j);
        for i in 0..w {
            let c = Color::new(i as f64 / w as f64, j as f64 / h as f64, 0.0);
            c.print_ppm();
        }
    }

    eprintln!("Done");
}
