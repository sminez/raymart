use crate::{P3, V3};
use rand::random_range;

#[derive(Debug, Clone, Copy)]
pub struct Perlin<const N: usize = 256> {
    rand_vec: [V3; N],
    perm_x: [usize; N],
    perm_y: [usize; N],
    perm_z: [usize; N],
}

impl Default for Perlin {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Perlin<N> {
    pub fn new() -> Self {
        let mut rand_vec = [V3::default(); N];
        let mut perm_x = [0; N];
        let mut perm_y = [0; N];
        let mut perm_z = [0; N];

        for i in 0..N {
            rand_vec[i] = V3::random(-1.0, 1.0).unit_vector();
            for s in [&mut perm_x, &mut perm_y, &mut perm_z] {
                s[i] = i;
            }
        }

        for s in [&mut perm_x, &mut perm_y, &mut perm_z] {
            for i in (N - 1)..0 {
                let target = random_range(0..i);
                s.swap(i, target);
            }
        }

        Self {
            rand_vec,
            perm_x,
            perm_y,
            perm_z,
        }
    }

    pub fn noise(&self, p: P3) -> f64 {
        let u = p.x - p.x.floor();
        let v = p.y - p.y.floor();
        let w = p.z - p.z.floor();

        let i = p.x.floor() as isize;
        let j = p.y.floor() as isize;
        let k = p.z.floor() as isize;

        let mut c = [[[V3::default(); 2]; 2]; 2];

        #[allow(clippy::needless_range_loop)]
        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    c[di][dj][dk] = self.rand_vec[self.perm_x[((i + di as isize) & 255) as usize]
                        ^ self.perm_y[((j + dj as isize) & 255) as usize]
                        ^ self.perm_z[((k + dk as isize) & 255) as usize]];
                }
            }
        }

        let uu = u * u * (3.0 - 2.0 * u);
        let vv = v * v * (3.0 - 2.0 * v);
        let ww = w * w * (3.0 - 2.0 * w);
        let mut acc = 0.0;

        #[allow(clippy::needless_range_loop)]
        for i in 0..2 {
            let fi = i as f64;
            for j in 0..2 {
                let fj = j as f64;
                for k in 0..2 {
                    let fk = k as f64;
                    let weight = V3::new(u - fi, v - fj, w - fk);
                    acc += (fi * uu + (1.0 - fi) * (1.0 - uu))
                        * (fj * vv + (1.0 - fj) * (1.0 - vv))
                        * (fk * ww + (1.0 - fk) * (1.0 - ww))
                        * c[i][j][k].dot(&weight);
                }
            }
        }

        acc
    }

    pub fn turb(&self, p: P3, depth: usize) -> f64 {
        let mut acc = 0.0;
        let mut temp_p = p;
        let mut weight = 1.0;

        for _ in 0..depth {
            acc += weight * self.noise(temp_p);
            weight *= 0.5;
            temp_p *= 2.0;
        }

        acc.abs()
    }
}
