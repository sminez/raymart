use crate::{bvh::Bvh, hit::Hittable, material::Material, noise::Perlin, shapes::Triangle, P3, V3};
use std::{collections::HashMap, f32::consts::PI, mem};

#[derive(Debug, Clone)]
pub struct SphereMesh {
    center: P3,
    radius: f32,
    vertices: Vec<P3>,
    triangles: Vec<[usize; 3]>,
    mat: &'static Material,
}

impl SphereMesh {
    pub fn new(
        center: P3,
        radius: f32,
        directions: Vec<V3>,
        triangles: Vec<[usize; 3]>,
        mat: &'static Material,
    ) -> Self {
        let radius = radius.max(0.0);
        let vertices = directions
            .into_iter()
            .map(|dir| center + radius * dir)
            .collect();

        Self {
            center,
            radius,
            vertices,
            triangles,
            mat,
        }
    }

    pub fn map_vertices<F>(&mut self, mut f: F)
    where
        F: FnMut(P3, f32, &mut P3),
    {
        self.vertices
            .iter_mut()
            .for_each(|v| (f)(self.center, self.radius, v));
    }

    pub fn into_mesh(self) -> Hittable {
        let hittables: Vec<_> = self
            .triangles
            .into_iter()
            .flat_map(|[i, j, k]| {
                try_triangle(
                    self.center,
                    self.vertices[i],
                    self.vertices[j],
                    self.vertices[k],
                    self.mat,
                )
            })
            .map(Hittable::from)
            .collect();

        Hittable::Bvh(Bvh::new(hittables))
    }

    /// See "The UV Sphere" in https://danielsieger.com/blog/2021/03/27/generating-spheres.html
    ///
    /// Directions are computed on the unit sphere and then later mapped to the correct vertices
    /// using the provided center and radius
    pub fn uv_sphere(
        center: P3,
        radius: f32,
        n_lat: usize,
        n_lon: usize,
        mat: &'static Material,
    ) -> Self {
        let mut directions = Vec::with_capacity(2 + (n_lat - 1) * n_lon);
        let mut triangles = Vec::new();

        // north pole
        directions.push(V3::Y);

        // generate vertices for each lat/lon slice and stack
        for i in 1..n_lat {
            let phi = PI * i as f32 / n_lat as f32;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            for j in 0..n_lon {
                let theta = 2.0 * PI * j as f32 / n_lon as f32;
                directions.push(V3::new(
                    sin_phi * theta.cos(),
                    cos_phi,
                    sin_phi * theta.sin(),
                ));
            }
        }

        // south pole
        let south_pole = directions.len();
        directions.push(-V3::Y);

        // add triangle fans for each pole
        for i in 0..n_lon {
            // north
            let va = (i + 1) % n_lon + 1;
            let vb = i + 1;
            triangles.push([0, va, vb]);

            // south
            let va = i + n_lon * (n_lat - 2) + 1;
            let vb = (i + 1) % n_lon + n_lon * (n_lat - 2) + 1;
            triangles.push([south_pole, va, vb]);
        }

        // add triangles for each slice (two per quad)
        for i in 0..(n_lat - 2) {
            let i0 = i * n_lon + 1;
            let i1 = i0 + n_lon;

            for j0 in 0..n_lon {
                let j1 = (j0 + 1) % n_lon;
                let a = i0 + j0;
                let b = i0 + j1;
                let c = i1 + j0;
                let d = i1 + j1;

                triangles.push([a, c, b]);
                triangles.push([b, c, d]);
            }
        }

        Self::new(center, radius, directions, triangles, mat)
    }

    /// See "The Icophere" in https://danielsieger.com/blog/2021/03/27/generating-spheres.html
    ///
    /// Directions are computed on the unit sphere and then later mapped to the correct vertices
    /// using the provided center and radius
    pub fn icosphere(center: P3, radius: f32, subdivisions: usize, mat: &'static Material) -> Self {
        let mut mesh = TriangleMesh::unit_icosahedron();
        mesh.subdivide(subdivisions);

        Self::new(center, radius, mesh.directions, mesh.triangles, mat)
    }

    pub fn noise_sphere(
        center: P3,
        radius: f32,
        subdivisions: usize,
        frac_inv: f32,
        noise_depth: usize,
        mat: &'static Material,
    ) -> Self {
        let mut mesh = TriangleMesh::unit_icosahedron();
        mesh.subdivide(subdivisions);
        mesh.apply_noise(frac_inv, noise_depth, &Perlin::new_from_thread_rng());

        Self::new(center, radius, mesh.directions, mesh.triangles, mat)
    }
}

struct TriangleMesh {
    directions: Vec<V3>,
    triangles: Vec<[usize; 3]>,
}

impl TriangleMesh {
    /// See https://danielsieger.com/blog/2021/01/03/generating-platonic-solids.html
    fn unit_icosahedron() -> Self {
        let phi = (1.0 + (5.0f32).sqrt()) / 2.0;

        // normalize is being used here to ensure that these are all points on the unit sphere
        let directions = vec![
            V3::new(-1.0, phi, 0.0).normalize(),
            V3::new(1.0, phi, 0.0).normalize(),
            V3::new(-1.0, -phi, 0.0).normalize(),
            V3::new(1.0, -phi, 0.0).normalize(),
            V3::new(0.0, -1.0, phi).normalize(),
            V3::new(0.0, 1.0, phi).normalize(),
            V3::new(0.0, -1.0, -phi).normalize(),
            V3::new(0.0, 1.0, -phi).normalize(),
            V3::new(phi, 0.0, -1.0).normalize(),
            V3::new(phi, 0.0, 1.0).normalize(),
            V3::new(-phi, 0.0, -1.0).normalize(),
            V3::new(-phi, 0.0, 1.0).normalize(),
        ];

        let triangles = vec![
            [0, 11, 5],
            [0, 5, 1],
            [0, 1, 7],
            [0, 7, 10],
            [0, 10, 11],
            [1, 5, 9],
            [5, 11, 4],
            [11, 10, 2],
            [10, 7, 6],
            [7, 1, 8],
            [3, 9, 4],
            [3, 4, 2],
            [3, 2, 6],
            [3, 6, 8],
            [3, 8, 9],
            [4, 9, 5],
            [2, 4, 11],
            [6, 2, 10],
            [8, 6, 7],
            [9, 8, 1],
        ];

        Self {
            directions,
            triangles,
        }
    }

    fn subdivide(&mut self, subdivisions: usize) {
        let (mut dirs, mut triangles) = (
            mem::take(&mut self.directions),
            mem::take(&mut self.triangles),
        );
        let mut cache = HashMap::new();

        let mut midpoint = |a, b, cache: &mut HashMap<(usize, usize), usize>| {
            let key = if a < b { (a, b) } else { (b, a) };
            if let Some(&idx) = cache.get(&key) {
                return idx;
            }

            let midpoint = (dirs[a] + dirs[b]).normalize();
            dirs.push(midpoint);
            let idx = dirs.len() - 1;
            cache.insert(key, idx);

            idx
        };

        for _ in 0..subdivisions {
            let mut next = Vec::with_capacity(triangles.len() * 4);
            cache.clear();

            for [a, b, c] in triangles {
                let ab = midpoint(a, b, &mut cache);
                let bc = midpoint(b, c, &mut cache);
                let ca = midpoint(c, a, &mut cache);

                next.push([a, ab, ca]);
                next.push([b, bc, ab]);
                next.push([c, ca, bc]);
                next.push([ab, bc, ca]);
            }

            triangles = next;
        }

        self.directions = dirs;
        self.triangles = triangles;
    }

    fn apply_noise(&mut self, frac_inv: f32, depth: usize, noise: &Perlin<256>) {
        let weights: Vec<f32> = self
            .directions
            .iter()
            .map(|dir| noise.turb(*dir, depth))
            .collect();

        let mut max = weights
            .iter()
            .fold(0.0, |acc, &val| if val > acc { val } else { acc });

        let base = 1.0 - (1.0 / frac_inv);
        max *= frac_inv;

        for (d, w) in self.directions.iter_mut().zip(weights) {
            *d *= base + w / max;
        }
    }
}

fn try_triangle(center: P3, a: P3, b: P3, c: P3, mat: &'static Material) -> Option<Triangle> {
    let normal = (b - a).cross(c - a);
    if normal.length_squared() < 1e-12 {
        return None;
    }

    let centroid_dir = ((a + b + c) / 3.0) - center;
    let (b, c) = if normal.dot(centroid_dir) < 0.0 {
        (c, b)
    } else {
        (b, c)
    };

    Some(Triangle::new(a, b, c, mat))
}
