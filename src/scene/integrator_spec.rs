use crate::{
    camera::Camera,
    integrator::{DepthIntegrator, Integrator, RaycastIntegrator, SimpleIntegrator},
    leak_ptr,
    sampler::Sampler,
    Color,
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum IntegratorSpec {
    Simple {
        samples_per_pixel: usize,
        step_size: Option<usize>,
        max_bounces: u8,
    },
    Depth {
        max_depth: Option<f32>,
    },
    Raycast {
        max_depth: f32,
    },
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
        match self {
            Self::Simple {
                samples_per_pixel,
                step_size,
                max_bounces,
            } => {
                leak_ptr!(SimpleIntegrator::new(
                    camera,
                    sampler,
                    samples_per_pixel,
                    step_size.unwrap_or(0),
                    max_bounces,
                    bg,
                ))
            }

            Self::Depth { max_depth } => {
                leak_ptr!(DepthIntegrator::new(camera, max_depth))
            }

            Self::Raycast { max_depth } => leak_ptr!(RaycastIntegrator::new(camera, max_depth)),
        }
    }
}
