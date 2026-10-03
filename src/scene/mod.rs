use crate::{
    hit::{Hittable, HittableList},
    integrator::Integrator,
    material::Material,
    sampler::SimpleSampler,
    Rng,
};
use rand::SeedableRng;
use serde::Deserialize;
use std::{collections::HashMap, fs};

mod camera_spec;
mod integrator_spec;
mod scene_spec;

pub use camera_spec::CameraSpec;
pub use integrator_spec::IntegratorSpec;
pub use scene_spec::SceneSpec;

#[derive(Debug)]
pub struct Scene {
    pub integrator: &'static mut dyn Integrator,
    pub hittables: Vec<&'static dyn Hittable>,
    pub lights: HittableList,
    pub materials: HashMap<String, &'static dyn Material>,
}

impl Scene {
    pub fn try_from_file(path: &str) -> anyhow::Result<Self> {
        let raw = RawScene::try_from_file(path)?;
        let mut rng = Rng::seed_from_u64(0);

        Self::try_from_raw(raw, &mut rng)
    }

    pub fn try_from_str(content: &str) -> anyhow::Result<Self> {
        let raw = RawScene::try_from_str(content)?;
        let mut rng = Rng::seed_from_u64(0);

        Self::try_from_raw(raw, &mut rng)
    }

    pub fn try_from_raw(raw: RawScene, rng: &mut Rng) -> anyhow::Result<Self> {
        let (hittables, lights, materials, bg) = raw.scene.into_scene(rng);
        let camera = raw.camera.into_simple_camera();
        let sampler = SimpleSampler::new(&camera, 0.5);
        let integrator = raw.integrator.into_integrator(camera, sampler, bg);

        Ok(Self {
            integrator,
            hittables,
            lights,
            materials,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawScene {
    pub integrator: IntegratorSpec,
    pub camera: CameraSpec,
    pub scene: SceneSpec,
}

impl RawScene {
    pub fn try_from_file(path: &str) -> anyhow::Result<Self> {
        let s = fs::read_to_string(path)?;

        Ok(serde_yaml::from_str(&s)?)
    }

    pub fn try_from_str(content: &str) -> anyhow::Result<Self> {
        Ok(serde_yaml::from_str(content)?)
    }
}
