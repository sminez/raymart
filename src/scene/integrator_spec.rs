use crate::{
    camera::Camera,
    integrator::{DepthIntegrator, Integrator, SimpleIntegrator},
    sampler::Sampler,
    Color,
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct IntegratorSpec {
    pub samples_per_pixel: usize,
    #[serde(default)]
    pub step_size: usize,
    pub max_bounces: u8,
    pub render_depth_map: bool,
}

impl IntegratorSpec {
    pub fn into_integrator<C, S>(
        self,
        camera: C,
        sampler: S,
        bg: Color,
    ) -> &'static mut dyn Integrator
    where
        C: Camera + 'static,
        S: Sampler + 'static,
    {
        if self.render_depth_map {
            Box::leak(Box::new(DepthIntegrator::new(camera)) as Box<dyn Integrator>)
        } else {
            Box::leak(Box::new(SimpleIntegrator::new(
                camera,
                sampler,
                self.samples_per_pixel,
                self.step_size,
                self.max_bounces,
                bg,
            )))
        }
    }
}
