//! helpers for working with meshes and scenes defined in config files
//!   https://docs.blender.org/manual/en/dev/modeling/meshes/introduction.html
//!   https://en.wikipedia.org/wiki/Wavefront_.obj_file
use crate::{
    bvh::Bvh,
    hit::{
        transforms::{ConstantMedium, Rotate, Translate},
        Hittable, HittableList,
    },
    leak_ptr,
    material::{Dielectric, DiffuseLight, Isotropic, Lambertian, Material, Metal, Specular},
    p,
    ray::Camera,
    shapes::{cuboid, Quad, Sphere, SphereMesh, Triangle},
    v, Color, Rng, DEBUG_SAMPLES_PER_PIXEL, DEFOCUS_ANGLE, FOCUS_DIST, IMAGE_WIDTH, MAX_BOUNCES,
    P3, STEP_SIZE, V3,
};
use glam::Mat3;
use serde::Deserialize;
use std::{collections::HashMap, fs};
use tobj::{load_obj, GPU_LOAD_OPTIONS};

macro_rules! pt {
    ($ps:expr, $ix:expr, $i: expr) => {{
        let idx = $ix[$i] as usize * 3;
        P3::new($ps[idx], $ps[idx + 1], $ps[idx + 2])
    }};
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(untagged)]
pub enum ColorSpec {
    RGB([f32; 3]),
    Grey(f32),
}

impl From<&ColorSpec> for Color {
    fn from(value: &ColorSpec) -> Self {
        match *value {
            ColorSpec::RGB([r, g, b]) => Color::new(r, g, b),
            ColorSpec::Grey(v) => V3::splat(v),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum MatSpec {
    Solid {
        color: ColorSpec,
    },
    Specular {
        color: ColorSpec,
        smoothness: f32,
        spec_prob: f32,
    },
    Checker {
        scale: f32,
        even: ColorSpec,
        odd: ColorSpec,
    },
    Metal {
        color: ColorSpec,
        fuzz: f32,
    },
    Dielectric {
        ref_index: f32,
        #[serde(default)]
        color: Option<ColorSpec>,
    },
    Isotropic {
        color: ColorSpec,
    },
    Light {
        color: ColorSpec,
    },
    Noise {
        scale: f32,
        #[serde(default)]
        color: Option<ColorSpec>,
    },
    Image {
        path: String,
    },
}

impl MatSpec {
    fn as_color(&self) -> Color {
        match self {
            Self::Solid { color } => color.into(),
            Self::Metal { color, .. } => color.into(),
            Self::Isotropic { color, .. } => color.into(),
            Self::Light { color } => color.into(),
            _ => panic!("no color associated with material"),
        }
    }

    fn as_material(&self, rng: &mut Rng) -> &'static dyn Material {
        match self {
            MatSpec::Solid { color } => Lambertian::solid_color(color),
            MatSpec::Specular {
                color,
                smoothness,
                spec_prob,
            } => Specular::new_mat(color, *smoothness, *spec_prob),
            MatSpec::Checker { scale, odd, even } => Lambertian::checker(*scale, odd, even),
            MatSpec::Metal { color, fuzz } => Metal::new_mat(color, *fuzz),
            MatSpec::Dielectric { ref_index, color } => {
                Dielectric::new_mat(color.as_ref().unwrap_or(&ColorSpec::Grey(1.0)), *ref_index)
            }
            MatSpec::Isotropic { color } => Isotropic::new_color(color),
            MatSpec::Light { color } => DiffuseLight::new_color(color),
            MatSpec::Noise { scale, color } => {
                Lambertian::noise(*scale, color.as_ref().unwrap_or(&ColorSpec::Grey(0.5)), rng)
            }
            MatSpec::Image { path } => Lambertian::image(path),
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct HitMeta {
    #[serde(default)]
    rotate: Option<f32>,
    #[serde(default)]
    translate: Option<[f32; 3]>,
    #[serde(default)]
    density: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Mesh {
    pub path: String,
    pub material: String,
    #[serde(default)]
    pub scale: f32,
    #[serde(flatten)]
    pub meta: HitMeta,
}

impl Mesh {
    fn color(&self, mats: &HashMap<String, MatSpec>) -> Color {
        mats.get(&self.material).unwrap().as_color()
    }

    fn as_dyn_hittable(
        &self,
        mats: &HashMap<String, &'static dyn Material>,
        mat_specs: &HashMap<String, MatSpec>,
        as_points: bool,
        point_radius: f32,
    ) -> &'static dyn Hittable {
        let (models, _) = load_obj(&self.path, &GPU_LOAD_OPTIONS).unwrap();
        let mat = *mats.get(&self.material).unwrap();
        let mut objects = Vec::with_capacity(models.iter().map(|m| m.mesh.indices.len()).sum());
        let scale = if self.scale == 0.0 { 1.0 } else { self.scale };

        let rotation = self
            .meta
            .rotate
            .map(|angle| Mat3::from_rotation_y(angle.to_radians()));
        let translation = self.meta.translate.map(V3::from);

        eprintln!("Loading meshes from {:?}...", self.path);
        for m in models {
            eprintln!("  mesh name = {:?}", m.name);
            let ps = &m.mesh.positions;
            let ix = &m.mesh.indices;

            for i in 0..ix.len() / 3 {
                let mut a = pt!(ps, ix, i * 3) * scale;
                let mut b = pt!(ps, ix, i * 3 + 1) * scale;
                let mut c = pt!(ps, ix, i * 3 + 2) * scale;

                if let Some(rotation) = rotation {
                    a = rotation * a;
                    b = rotation * b;
                    c = rotation * c;
                }
                if let Some(offset) = translation {
                    a += offset;
                    b += offset;
                    c += offset;
                }

                if as_points {
                    objects.extend([a, b, c].into_iter().map(|p| {
                        leak_ptr!(Sphere::new(p, point_radius, mat)) as &'static dyn Hittable
                    }));
                } else {
                    objects.push(leak_ptr!(Triangle::new(a, b, c, mat)));
                }
            }

            eprintln!("    n vertices  = {}", ix.len());
            eprintln!("    n hittables = {}", objects.len());
        }

        let mut h: &'static dyn Hittable = leak_ptr!(Bvh::new(objects));

        if let Some(density) = self.meta.density {
            h = leak_ptr!(ConstantMedium::new(h, density, self.color(mat_specs)));
        }

        h
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ObjSpec {
    #[serde(flatten)]
    hittable: HittableSpec,
    #[serde(flatten)]
    pub meta: HitMeta,
}

impl ObjSpec {
    fn as_dyn_hittable(
        &self,
        mats: &HashMap<String, &'static dyn Material>,
        mat_specs: &HashMap<String, MatSpec>,
    ) -> &'static dyn Hittable {
        let mut h = self.hittable.as_dyn_hittable(mats);

        if let Some(angle) = self.meta.rotate {
            h = leak_ptr!(Rotate::new(h, angle));
        }
        if let Some(v) = self.meta.translate {
            h = leak_ptr!(Translate::new(h, v.into()));
        }
        if let Some(density) = self.meta.density {
            h = leak_ptr!(ConstantMedium::new(
                h,
                density,
                self.hittable.color(mat_specs)
            ));
        }

        h
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum HittableSpec {
    Sphere {
        center: [f32; 3],
        r: f32,
        material: String,
    },
    UvSphere {
        center: [f32; 3],
        r: f32,
        n_lat: usize,
        n_lon: usize,
        material: String,
    },
    IcoSphere {
        center: [f32; 3],
        r: f32,
        subdivisions: usize,
        material: String,
    },
    NoiseSphere {
        center: [f32; 3],
        r: f32,
        subdivisions: usize,
        subtract: bool,
        frac_inv: f32,
        noise_depth: usize,
        #[serde(default)]
        seed: Option<String>,
        material: String,
    },
    Box {
        vert1: [f32; 3],
        vert2: [f32; 3],
        material: String,
    },
    Quad {
        q: [f32; 3],
        u: [f32; 3],
        v: [f32; 3],
        material: String,
    },
    Triangle {
        a: [f32; 3],
        b: [f32; 3],
        c: [f32; 3],
        material: String,
    },
}

impl HittableSpec {
    fn color(&self, mats: &HashMap<String, MatSpec>) -> Color {
        let mat = match self {
            Self::Sphere { material, .. } => mats.get(material).unwrap(),
            Self::UvSphere { material, .. } => mats.get(material).unwrap(),
            Self::IcoSphere { material, .. } => mats.get(material).unwrap(),
            Self::NoiseSphere { material, .. } => mats.get(material).unwrap(),
            Self::Box { material, .. } => mats.get(material).unwrap(),
            Self::Quad { material, .. } => mats.get(material).unwrap(),
            Self::Triangle { material, .. } => mats.get(material).unwrap(),
        };

        mat.as_color()
    }

    fn material(&self) -> &str {
        match self {
            Self::Sphere { material, .. } => material,
            Self::UvSphere { material, .. } => material,
            Self::IcoSphere { material, .. } => material,
            Self::NoiseSphere { material, .. } => material,
            Self::Box { material, .. } => material,
            Self::Quad { material, .. } => material,
            Self::Triangle { material, .. } => material,
        }
    }

    fn as_dyn_hittable(
        &self,
        mats: &HashMap<String, &'static dyn Material>,
    ) -> &'static dyn Hittable {
        let mat = |material: &str| {
            *mats
                .get(material)
                .unwrap_or_else(|| panic!("unknown material: {material}"))
        };

        match self {
            Self::Sphere {
                center,
                r,
                material,
            } => {
                leak_ptr!(Sphere::new((*center).into(), *r, mat(material))) as &'static dyn Hittable
            }

            Self::UvSphere {
                center,
                r,
                n_lat,
                n_lon,
                material,
            } => SphereMesh::uv_sphere((*center).into(), *r, *n_lat, *n_lon, mat(material))
                .into_dyn_hittable(),

            Self::IcoSphere {
                center,
                r,
                subdivisions,
                material,
            } => SphereMesh::icosphere((*center).into(), *r, *subdivisions, mat(material))
                .into_dyn_hittable(),

            Self::NoiseSphere {
                center,
                r,
                subdivisions,
                subtract,
                frac_inv,
                noise_depth,
                seed,
                material,
            } => SphereMesh::noise_sphere(
                (*center).into(),
                *r,
                *subdivisions,
                *subtract,
                *frac_inv,
                *noise_depth,
                seed.as_deref(),
                mat(material),
            )
            .into_dyn_hittable(),

            Self::Box {
                vert1,
                vert2,
                material,
            } => leak_ptr!(cuboid((*vert1).into(), (*vert2).into(), mat(material))),

            Self::Quad { q, u, v, material } => {
                leak_ptr!(Quad::new(
                    (*q).into(),
                    (*u).into(),
                    (*v).into(),
                    mat(material)
                ))
            }

            Self::Triangle { a, b, c, material } => {
                leak_ptr!(Triangle::new(
                    (*a).into(),
                    (*b).into(),
                    (*c).into(),
                    mat(material)
                ))
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Scene {
    // sim
    pub samples_per_pixel: u16,
    #[serde(default)]
    pub samples_step_size: u16,
    pub max_bounces: u8,
    // camera
    pub fov: f32,
    pub image_width: u16,
    pub aspect_ratio: f32,
    pub from: [f32; 3],
    pub at: [f32; 3],
    pub v_up: [f32; 3],
    #[serde(default)]
    pub defocus_angle: f32,
    #[serde(default = "default_focus_dist")]
    pub focus_dist: f32,
    // hittables
    pub as_points: bool,
    pub point_radius: f32,
    pub materials: HashMap<String, MatSpec>,
    #[serde(default)]
    pub meshes: Vec<Mesh>,
    #[serde(default)]
    pub objects: Vec<ObjSpec>,
    // light
    pub bg: ColorSpec,
}

fn default_focus_dist() -> f32 {
    FOCUS_DIST
}

impl Default for Scene {
    fn default() -> Self {
        Scene {
            samples_per_pixel: DEBUG_SAMPLES_PER_PIXEL,
            samples_step_size: STEP_SIZE,
            max_bounces: MAX_BOUNCES,
            image_width: IMAGE_WIDTH,
            aspect_ratio: 1.0,
            fov: 40.0,
            from: [1.2, 0.2, -0.85],
            at: [0.0, 0.0, 0.0],
            v_up: [0.0, 1.0, 0.0],
            as_points: false,
            defocus_angle: DEFOCUS_ANGLE,
            focus_dist: FOCUS_DIST,
            point_radius: 0.001,
            materials: [
                (
                    "grey",
                    MatSpec::Solid {
                        color: ColorSpec::Grey(0.5),
                    },
                ),
                (
                    "light",
                    MatSpec::Light {
                        color: ColorSpec::Grey(25.0),
                    },
                ),
            ]
            .into_iter()
            .map(|(s, m)| (s.to_string(), m))
            .collect(),
            meshes: vec![Mesh {
                path: "assets/Dragon_8K.obj".to_string(),
                material: "grey".to_string(),
                scale: 1.0,
                meta: HitMeta::default(),
            }],
            objects: vec![ObjSpec {
                hittable: HittableSpec::Sphere {
                    center: [1.0, 1.0, 1.0],
                    r: 1.0,
                    material: "light".to_string(),
                },
                meta: HitMeta::default(),
            }],
            bg: ColorSpec::RGB([0.7, 0.8, 1.0]),
        }
    }
}

impl Scene {
    pub fn try_from_file(path: &str) -> Option<Self> {
        let s = fs::read_to_string(path).ok()?;

        Some(toml::from_str(&s).unwrap())
    }

    pub fn try_from_str(content: &str) -> Option<Self> {
        Some(toml::from_str(content).unwrap())
    }

    pub fn load_scene(&self, rng: &mut Rng) -> (Vec<&'static dyn Hittable>, HittableList, Camera) {
        let mut hittables = Vec::new();
        let lights_names: Vec<_> = self
            .materials
            .iter()
            .filter_map(|(k, v)| {
                if matches!(v, MatSpec::Light { .. }) {
                    Some(k.as_str())
                } else {
                    None
                }
            })
            .collect();

        let materials: HashMap<String, &'static dyn Material> = self
            .materials
            .iter()
            .map(|(k, v)| (k.clone(), v.as_material(rng)))
            .collect();

        let mut lights = HittableList::default();

        for mesh in self.meshes.iter() {
            let h = mesh.as_dyn_hittable(
                &materials,
                &self.materials,
                self.as_points,
                self.point_radius,
            );

            if lights_names.contains(&mesh.material.as_str()) {
                lights.add(h);
            }

            hittables.push(h);
        }

        for obj in self.objects.clone().into_iter() {
            let h = obj.as_dyn_hittable(&materials, &self.materials);
            if lights_names.contains(&obj.hittable.material()) {
                lights.add(h);
            }

            hittables.push(h);
        }

        let v_up = v!(self.v_up[0], self.v_up[1], self.v_up[2]);
        let look_from = p!(self.from[0], self.from[1], self.from[2]);
        let look_at = p!(self.at[0], self.at[1], self.at[2]);

        let camera = Camera::new(
            self.aspect_ratio,
            self.image_width,
            self.samples_per_pixel,
            self.samples_step_size,
            self.max_bounces,
            (&self.bg).into(),
            self.fov,
            look_from,
            look_at,
            v_up,
            self.defocus_angle,
            self.focus_dist,
        );

        (hittables, lights, camera)
    }
}
