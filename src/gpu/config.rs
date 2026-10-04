use bytemuck::{Pod, Zeroable};
use eframe::egui::Vec2;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct ShaderConfig {
    pub scale: f32,
    pub escape_threshold: f32,
    pub centre: [f32; 2],
    pub iterations: i32,
    pub flags: u32,
    // Which component indices (0..=15) map to screen x and y respectively.
    pub axis_x: u32,
    pub axis_y: u32,
    pub fixed: [f32; 16],
    // Julia: the fixed constant c. Mandelbrot: initial z.
    pub initial_value: [f32; 16],
}

#[derive(Debug, Clone)]
pub struct Config {
    pub zoom: f32,
    pub centre: [f32; 2],
    pub iterations: i32,
    pub julia_set: bool,
    pub internal_black: bool,
    pub initial_value: [f32; 16],
    pub axis_x: u32,
    pub axis_y: u32,
    pub fixed: [f32; 16],
    pub escape: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            centre: [0.0, 0.0],
            iterations: 100,
            julia_set: false,
            internal_black: false,
            initial_value: [0.0; 16],
            axis_x: 0,
            axis_y: 1,
            fixed: [0.0; 16],
            escape: 2.0,
        }
    }
}

impl Config {
    pub fn scale(&self, size: Vec2) -> f32 {
        4.0 / self.zoom / size.min_elem()
    }

    pub fn as_shader_config(&self, size: Vec2) -> ShaderConfig {
        let scale = self.scale(size);

        ShaderConfig {
            scale,
            centre: [
                size.x / 2.0 * scale - self.centre[0],
                size.y / 2.0 * scale - self.centre[1],
            ],
            iterations: self.iterations,
            flags: (self.internal_black as u32) << 1 | (self.julia_set as u32),
            axis_x: self.axis_x,
            axis_y: self.axis_y,
            fixed: self.fixed,
            initial_value: self.initial_value,
            escape_threshold: self.escape,
        }
    }
}
