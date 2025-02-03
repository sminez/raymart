use crate::P3;
use rand::random_range;

#[derive(Debug, Clone, Copy)]
pub struct Perlin<const N: usize = 256> {
    rand_float: [f64; N],
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
        let mut rand_float = [0.0; N];
        let mut perm_x = [0; N];
        let mut perm_y = [0; N];
        let mut perm_z = [0; N];

        for i in 0..N {
            rand_float[i] = random_range(0.0..1.0);
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
            rand_float,
            perm_x,
            perm_y,
            perm_z,
        }
    }

    pub fn noise(&self, p: P3) -> f64 {
        let i = ((4.0 * p.x) as isize & 255) as usize;
        let j = ((4.0 * p.y) as isize & 255) as usize;
        let k = ((4.0 * p.z) as isize & 255) as usize;

        self.rand_float[self.perm_x[i] ^ self.perm_y[j] ^ self.perm_z[k]]
    }
}
