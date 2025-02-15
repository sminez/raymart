//! helpers for working with meshes and scenes defined in config files
//!   https://docs.blender.org/manual/en/dev/modeling/meshes/introduction.html
//!   https://en.wikipedia.org/wiki/Wavefront_.obj_file
use crate::{
    bvh::Bvh,
    hit::{cuboid, ConstantMedium, Hittable, Quad, Sphere, Triangle},
    material::Material,
    p,
    ray::Camera,
    v, Color, DEBUG_SAMPLES_PER_PIXEL, IMAGE_WIDTH, MAX_BOUNCES, P3, STEP_SIZE, V3,
};
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
            ColorSpec::Grey(v) => Color::grey(v),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase", tag = "kind")]
pub enum MatSpec {
    Solid {
        color: ColorSpec,
    },
    Specular {
        color: ColorSpec,
        spec_color: ColorSpec,
        smoothness: f32,
        spec_prob: f32,
    },
    Checker {
        scale: f32,
        odd: ColorSpec,
        even: ColorSpec,
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
}

impl From<&MatSpec> for Material {
    fn from(m: &MatSpec) -> Self {
        match m {
            MatSpec::Solid { color } => Material::solid_color(color.into()),
            MatSpec::Specular {
                color,
                spec_color,
                smoothness,
                spec_prob,
            } => Material::Specular {
                albedo: color.into(),
                spec_albedo: spec_color.into(),
                smoothness: *smoothness,
                prob: *spec_prob,
            },
            MatSpec::Checker { scale, odd, even } => {
                Material::checker(*scale, even.into(), odd.into())
            }
            MatSpec::Metal { color, fuzz } => Material::metal(color.into(), *fuzz),
            MatSpec::Dielectric { ref_index, color } => Material::dielectric(
                *ref_index,
                color.as_ref().unwrap_or(&ColorSpec::Grey(1.0)).into(),
            ),
            MatSpec::Isotropic { color } => Material::isotropic(color.into()),
            MatSpec::Light { color } => Material::diffuse_light(color.into()),
            MatSpec::Noise { scale } => Material::noise(*scale),
            MatSpec::Image { path } => Material::image(path),
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

    fn as_hittable(
        &self,
        mats: &HashMap<String, &'static Material>,
        mat_specs: &HashMap<String, MatSpec>,
        as_points: bool,
        point_radius: f32,
    ) -> Hittable {
        let (models, _) = load_obj(&self.path, &GPU_LOAD_OPTIONS).unwrap();
        let mat = *mats.get(&self.material).unwrap();
        let mut objects = Vec::with_capacity(models.iter().map(|m| m.mesh.indices.len()).sum());
        let scale = if self.scale == 0.0 { 1.0 } else { self.scale };

        eprintln!("Loading meshes from {:?}...", self.path);
        for m in models {
            eprintln!("  mesh name = {:?}", m.name);
            let ps = &m.mesh.positions;
            let ix = &m.mesh.indices;

            for i in 0..ix.len() / 3 {
                let mut a = pt!(ps, ix, i * 3) * scale;
                let mut b = pt!(ps, ix, i * 3 + 1) * scale;
                let mut c = pt!(ps, ix, i * 3 + 2) * scale;

                if let Some(angle) = self.meta.rotate {
                    let rad = angle.to_radians();
                    let sin_theta = rad.sin();
                    let cos_theta = rad.cos();

                    for v in [&mut a, &mut b, &mut c] {
                        *v = V3::new(
                            cos_theta * v.x + sin_theta * v.z,
                            v.y,
                            -sin_theta * v.x + cos_theta * v.z,
                        );
                    }
                }

                if let Some(v) = self.meta.translate {
                    let v: V3 = v.into();
                    a += v;
                    b += v;
                    c += v;
                }

                if as_points {
                    objects.extend(
                        [a, b, c]
                            .into_iter()
                            .map(|p| Hittable::from(Sphere::new(p, point_radius, mat))),
                    );
                } else {
                    objects.push(Triangle::new(a, b, c, mat).into());
                }
            }

            eprintln!("    n vertices  = {}", ix.len());
            eprintln!("    n hittables = {}", objects.len());
        }

        let mut h = Hittable::Bvh(Bvh::new(objects));

        if let Some(density) = self.meta.density {
            h = ConstantMedium::new(h, density, self.color(mat_specs)).into();
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
    fn as_hittable(
        &self,
        mats: &HashMap<String, &'static Material>,
        mat_specs: &HashMap<String, MatSpec>,
    ) -> Hittable {
        let mut h = self.hittable.as_hittable(mats);
        if let Some(angle) = self.meta.rotate {
            h = h.rotate(angle);
        }
        if let Some(v) = self.meta.translate {
            h = h.translate(v.into());
        }
        if let Some(density) = self.meta.density {
            h = ConstantMedium::new(h, density, self.hittable.color(mat_specs)).into();
        }

        h
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase", tag = "kind")]
pub enum HittableSpec {
    Sphere {
        center: [f32; 3],
        r: f32,
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
            Self::Box { material, .. } => mats.get(material).unwrap(),
            Self::Quad { material, .. } => mats.get(material).unwrap(),
            Self::Triangle { material, .. } => mats.get(material).unwrap(),
        };

        mat.as_color()
    }

    fn as_hittable(&self, mats: &HashMap<String, &'static Material>) -> Hittable {
        let mat = |material: &str| {
            mats.get(material)
                .unwrap_or_else(|| panic!("unknown material: {material}"))
        };

        match self {
            Self::Sphere {
                center,
                r,
                material,
            } => Sphere::new((*center).into(), *r, mat(material)).into(),

            Self::Box {
                vert1,
                vert2,
                material,
            } => cuboid((*vert1).into(), (*vert2).into(), mat(material)),

            Self::Quad { q, u, v, material } => {
                Quad::new((*q).into(), (*u).into(), (*v).into(), mat(material)).into()
            }

            Self::Triangle { a, b, c, material } => {
                Triangle::new((*a).into(), (*b).into(), (*c).into(), mat(material)).into()
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

    pub fn load_scene(&self) -> (Vec<Hittable>, Camera) {
        let mut hittables = Vec::new();
        let materials: HashMap<String, &'static Material> = self
            .materials
            .iter()
            .map(|(k, v)| (k.clone(), Box::leak(Box::new(v.into())) as &'static _))
            .collect();

        for mesh in self.meshes.iter() {
            hittables.push(mesh.as_hittable(
                &materials,
                &self.materials,
                self.as_points,
                self.point_radius,
            ));
        }

        for obj in self.objects.clone().into_iter() {
            hittables.push(obj.as_hittable(&materials, &self.materials));
        }

        let v_up = v!(self.v_up[0], self.v_up[1], self.v_up[2]);
        let defocus_angle = 0.0;
        let focus_dist = 10.0;
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
            defocus_angle,
            focus_dist,
        );

        (hittables, camera)
    }
}
