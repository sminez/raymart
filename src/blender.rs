//! helpers for working with files from Blender
//!   https://docs.blender.org/manual/en/dev/modeling/meshes/introduction.html
//!   https://en.wikipedia.org/wiki/Wavefront_.obj_file
use crate::{
    bbox::AABBox,
    hit::{HitRecord, Hittable, Interval, Sphere},
    material::Material,
    p,
    ray::Camera,
    v, Ray, DEBUG_SAMPLES_PER_PIXEL, IMAGE_WIDTH, MAX_BOUNCES, P3, V3,
};
use serde::Deserialize;
use std::fs;
use tobj::{load_obj, GPU_LOAD_OPTIONS};

macro_rules! pt {
    ($ps:expr, $ix:expr, $i: expr) => {{
        let idx = $ix[$i] as usize * 3;
        P3::new($ps[idx], $ps[idx + 1], $ps[idx + 2])
    }};
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase", tag = "kind")]
pub enum MatSpec {
    Solid { color: [f64; 3] },
    Metal { color: [f64; 3], fuzz: f64 },
    Glass { ref_index: f64 },
    Isotropic { color: [f64; 3] },
    Light { color: [f64; 3] },
}

impl From<MatSpec> for Material {
    fn from(m: MatSpec) -> Self {
        match m {
            MatSpec::Solid { color } => Material::solid_color(color.into()),
            MatSpec::Metal { color, fuzz } => Material::metal(color.into(), fuzz),
            MatSpec::Glass { ref_index } => Material::dielectric(ref_index),
            MatSpec::Isotropic { color } => Material::isotropic(color.into()),
            MatSpec::Light { color } => Material::diffuse_light(color.into()),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Mesh {
    pub path: String,
    pub material: MatSpec,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase", tag = "kind")]
pub enum ObjSpec {
    Sphere {
        center: [f64; 3],
        r: f64,
        material: MatSpec,
    },
}

impl From<ObjSpec> for Hittable {
    fn from(obj: ObjSpec) -> Self {
        match obj {
            ObjSpec::Sphere {
                center,
                r,
                material,
            } => Sphere::new(center.into(), r, material.into()).into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Scene {
    // sim
    pub samples_per_pixel: u16,
    pub max_bounces: u8,
    // camera
    pub fov: f64,
    pub image_width: u16,
    pub aspect_ratio: f64,
    pub from: [f64; 3],
    pub at: [f64; 3],
    // hittables
    pub as_points: bool,
    pub point_radius: f64,
    pub meshes: Vec<Mesh>,
    pub objects: Vec<ObjSpec>,
    // light
    pub bg: [f64; 3],
}

impl Default for Scene {
    fn default() -> Self {
        Scene {
            samples_per_pixel: DEBUG_SAMPLES_PER_PIXEL,
            max_bounces: MAX_BOUNCES,
            image_width: IMAGE_WIDTH,
            aspect_ratio: 1.0,
            fov: 40.0,
            from: [1.2, 0.2, -0.85],
            at: [0.0, 0.0, 0.0],
            as_points: false,
            point_radius: 0.001,
            meshes: vec![Mesh {
                path: "assets/Dragon_8K.obj".to_string(),
                material: MatSpec::Solid {
                    color: [0.53, 0.53, 0.53],
                },
            }],
            objects: vec![ObjSpec::Sphere {
                center: [1.0, 1.0, 1.0],
                r: 1.0,
                material: MatSpec::Light {
                    color: [25.0, 25.0, 25.0],
                },
            }],
            bg: [0.7, 0.8, 1.0],
        }
    }
}

impl Scene {
    pub fn try_from_file() -> Option<Self> {
        let s = fs::read_to_string("scene.json").ok()?;

        Some(serde_json::from_str(&s).unwrap())
    }

    pub fn load_scene(&self) -> (Vec<Hittable>, Camera) {
        let mut hittables = Vec::new();

        for mesh in self.meshes.iter() {
            let (models, _) = load_obj(&mesh.path, &GPU_LOAD_OPTIONS).unwrap();
            let mat: Material = mesh.material.into();

            for m in models {
                let ps = &m.mesh.positions;
                let ix = &m.mesh.indices;

                for i in 0..ix.len() / 3 {
                    let a = pt!(ps, ix, i * 3);
                    let b = pt!(ps, ix, i * 3 + 1);
                    let c = pt!(ps, ix, i * 3 + 2);

                    if self.as_points {
                        hittables.extend(
                            [a, b, c]
                                .into_iter()
                                .map(|p| Hittable::from(Sphere::new(p, self.point_radius, mat))),
                        );
                    } else {
                        hittables.push(Triangle::new(a, b, c, mat).into());
                    }
                }

                eprintln!("n vertices  = {}", ix.len());
                eprintln!("n hittables = {}", hittables.len());
            }
        }

        for obj in self.objects.iter() {
            hittables.push((*obj).into());
        }

        let v_up = v!(0, 1, 0);
        let defocus_angle = 0.0;
        let focus_dist = 10.0;
        let look_from = p!(self.from[0], self.from[1], self.from[2]);
        let look_at = p!(self.at[0], self.at[1], self.at[2]);

        let camera = Camera::new(
            self.aspect_ratio,
            self.image_width,
            self.samples_per_pixel,
            self.max_bounces,
            v!(self.bg[0], self.bg[1], self.bg[2]),
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

#[derive(Debug, Clone)]
pub struct Triangle {
    a: P3,
    ab: V3,
    ac: V3,
    normal: V3,
    mat: Material,
    pub bbox: AABBox,
}

impl Triangle {
    pub fn new(a: P3, b: P3, c: P3, mat: Material) -> Triangle {
        let bbox1 = AABBox::new_from_points(a, b);
        let bbox2 = AABBox::new_from_points(a, c);
        let ab = b - a;
        let ac = c - a;
        let normal = ab.cross(&ac);

        Self {
            a,
            ab,
            ac,
            normal,
            mat,
            bbox: AABBox::new_enclosing(bbox1, bbox2),
        }
    }

    // Calculate the intersection of a ray with a triangle using the Möller–Trumbore algorithm
    //   https://en.wikipedia.org/wiki/M%C3%B6ller%E2%80%93Trumbore_intersection_algorithm
    pub fn hits(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        // If r . normal is 0 then the ray is parallel to the triangle plane and no hit is possible
        let det = -(r.dir.dot(&self.normal));
        if det.abs() < 1e-8 {
            return None;
        }

        let inv_det = 1.0 / det;
        let ao = r.orig - self.a;
        let r_x_ao = ao.cross(&r.dir);

        // hit point needs to be contained by the ray interval
        let t = ao.dot(&self.normal) * inv_det;
        if !ray_t.surrounds(t) {
            return None;
        }

        // barycentric coords of the intersection point
        //   https://en.wikipedia.org/wiki/Barycentric_coordinate_system
        let u = self.ac.dot(&r_x_ao) * inv_det;
        let v = -self.ab.dot(&r_x_ao) * inv_det;
        if u < 0.0 || v < 0.0 || u + v > 1.0 {
            return None;
        }

        let p = r.at(t);

        Some(HitRecord::new(t, p, self.normal, r, self.mat, u, v))
    }
}
