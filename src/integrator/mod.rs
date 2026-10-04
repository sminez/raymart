use crate::{camera::Camera, hit::Hittable, Bvh, Color};
use std::{fmt, mem, ops::ControlFlow};

mod depth;
mod normal;
mod raycast;
mod simple;

pub use depth::DepthIntegrator;
pub use normal::NormalIntegrator;
pub use raycast::RaycastIntegrator;
pub use simple::SimpleIntegrator;

pub trait Integrator: fmt::Debug {
    fn num_iterations(&self) -> usize;
    fn camera(&self) -> &dyn Camera;
    fn next_render_pass(
        &mut self,
        i: usize,
        bvh: &Bvh,
        lights: &'static dyn Hittable,
        current_pixels: &[Color],
        new_pixels: &mut Vec<Color>,
    );

    fn render(&mut self, bvh: Bvh, lights: &'static dyn Hittable) -> Vec<Color> {
        let (_, pixels) =
            render_with_hook(self, bvh, lights, |_, _| ControlFlow::<()>::Continue(()));

        pixels
    }
}

pub fn render_with_hook<I, F, T>(
    integrator: &mut I,
    bvh: Bvh,
    lights: &'static dyn Hittable,
    mut per_iteration: F,
) -> (Option<T>, Vec<Color>)
where
    I: Integrator + ?Sized,
    F: FnMut(usize, &[Color]) -> ControlFlow<T>,
{
    let (w, h) = integrator.camera().image_dims();
    let mut pixels = vec![Color::default(); w * h];
    let mut new_pixels = vec![Color::default(); w * h];

    let iterations = integrator.num_iterations();

    for i in 1..=iterations {
        integrator.next_render_pass(i, &bvh, lights, &pixels, &mut new_pixels);
        mem::swap(&mut pixels, &mut new_pixels);

        if let ControlFlow::Break(break_val) = per_iteration(i, &pixels) {
            return (Some(break_val), pixels);
        }
    }

    (None, pixels)
}
