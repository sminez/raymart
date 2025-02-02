//! A simple 3D vector using f64s
use rand::random_range;
use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

const NEAR_ZERO: f64 = 1e-8;

pub type P3 = V3;

#[derive(Debug, Default, Copy, Clone)]
pub struct V3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl V3 {
    pub const fn new(x: f64, y: f64, z: f64) -> V3 {
        Self { x, y, z }
    }

    pub fn random(min: f64, max: f64) -> V3 {
        V3::new(
            random_range(min..max),
            random_range(min..max),
            random_range(min..max),
        )
    }

    pub fn random_unit_vector() -> V3 {
        loop {
            let p = Self::random(-1.0, 1.0);
            let sq_len = p.square_length();
            if 1e-160 < sq_len && sq_len < 1.0 {
                return p / sq_len.sqrt(); // avoiding computing sq_len again
            }
        }
    }

    pub fn random_on_hemisphere(normal: &V3) -> V3 {
        let v = Self::random_unit_vector();
        if v.dot(normal) > 0.0 {
            v // Same hemisphere as `normal`
        } else {
            -v
        }
    }

    pub fn random_in_unit_disk() -> V3 {
        loop {
            let p = V3::new(random_range(-1.0..1.0), random_range(-1.0..1.0), 0.0);
            if p.square_length() < 1.0 {
                return p;
            }
        }
    }

    pub fn reflect(&self, normal: V3) -> V3 {
        *self - 2.0 * self.dot(&normal) * normal
    }

    pub fn refract(&self, normal: V3, etai_over_etat: f64) -> V3 {
        let cos_theta = (-self.dot(&normal)).min(1.0);
        let r_out_perp = etai_over_etat * (*self + cos_theta * normal);
        let r_out_para = -(1.0 - r_out_perp.square_length()).sqrt() * normal;

        r_out_perp + r_out_para
    }

    pub const fn dot(&self, rhs: &V3) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub const fn cross(&self, rhs: &V3) -> V3 {
        Self::new(
            self.y * rhs.z - self.z * rhs.y,
            self.z * rhs.x - self.x * rhs.z,
            self.x * rhs.y - self.y * rhs.x,
        )
    }

    pub fn length(&self) -> f64 {
        self.square_length().sqrt()
    }

    pub const fn square_length(&self) -> f64 {
        self.dot(self)
    }

    pub fn unit_vector(&self) -> V3 {
        *self / self.length()
    }

    pub fn near_zero(&self) -> bool {
        self.x.abs() < NEAR_ZERO && self.y.abs() < NEAR_ZERO && self.z.abs() < NEAR_ZERO
    }
}

impl Neg for V3 {
    type Output = V3;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl Add<V3> for V3 {
    type Output = V3;

    fn add(self, rhs: V3) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl AddAssign<V3> for V3 {
    fn add_assign(&mut self, rhs: V3) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub<V3> for V3 {
    type Output = V3;

    fn sub(self, rhs: V3) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl SubAssign<V3> for V3 {
    fn sub_assign(&mut self, rhs: V3) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl Mul<f64> for V3 {
    type Output = V3;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Mul<V3> for f64 {
    type Output = V3;

    fn mul(self, rhs: V3) -> Self::Output {
        rhs * self
    }
}

// This is odd...really this should be the full product so what is this used for???
impl Mul<V3> for V3 {
    type Output = V3;

    fn mul(self, rhs: V3) -> Self::Output {
        Self::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

impl MulAssign<f64> for V3 {
    fn mul_assign(&mut self, rhs: f64) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl Div<f64> for V3 {
    type Output = V3;

    fn div(self, rhs: f64) -> Self::Output {
        self * (1.0 / rhs)
    }
}

impl DivAssign<f64> for V3 {
    fn div_assign(&mut self, rhs: f64) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

impl Index<usize> for V3 {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("v3 index out of bounds: {index}"),
        }
    }
}

impl IndexMut<usize> for V3 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("v3 index out of bounds: {index}"),
        }
    }
}
