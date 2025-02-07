//! Axis-aligned bounding boxes and Bounding Volume Hierarchies
//! See Section 3 of https://raytracing.github.io/books/RayTracingTheNextWeek.html for the details

use crate::{
    hit::{HitRecord, Hittable, Interval},
    Ray, P3, V3,
};
use std::ops::Add;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct AABBox {
    pub x: Interval,
    pub y: Interval,
    pub z: Interval,
    min: wide::f64x4,
    max: wide::f64x4,
}

impl AABBox {
    pub const EMPTY: AABBox = AABBox::new(Interval::EMPTY, Interval::EMPTY, Interval::EMPTY);
    pub const UNIVERSE: AABBox =
        AABBox::new(Interval::UNIVERSE, Interval::UNIVERSE, Interval::UNIVERSE);

    pub const fn new(x: Interval, y: Interval, z: Interval) -> AABBox {
        let mut bbox = AABBox {
            x,
            y,
            z,
            min: wide::f64x4::ZERO,
            max: wide::f64x4::ZERO,
        };
        bbox.pad_to_minimum();

        bbox
    }

    pub const fn new_enclosing(a: AABBox, b: AABBox) -> AABBox {
        let mut bbox = AABBox {
            x: Interval::new_enclosing(a.x, b.x),
            y: Interval::new_enclosing(a.y, b.y),
            z: Interval::new_enclosing(a.z, b.z),
            min: wide::f64x4::ZERO,
            max: wide::f64x4::ZERO,
        };
        bbox.pad_to_minimum();

        bbox
    }

    /// Treat the two points a and b as extrema for the bounding box, so we don't require a
    /// particular minimum/maximum coordinate order.
    pub const fn new_from_points(a: P3, b: P3) -> AABBox {
        let (x1, x2) = if a.x <= b.x { (a.x, b.x) } else { (b.x, a.x) };
        let (y1, y2) = if a.y <= b.y { (a.y, b.y) } else { (b.y, a.y) };
        let (z1, z2) = if a.z <= b.z { (a.z, b.z) } else { (b.z, a.z) };

        let mut bbox = AABBox {
            x: Interval::new(x1, x2),
            y: Interval::new(y1, y2),
            z: Interval::new(z1, z2),
            min: wide::f64x4::ZERO,
            max: wide::f64x4::ZERO,
        };
        bbox.pad_to_minimum();

        bbox
    }

    pub fn hits(&self, r: &Ray, ray_t: Interval) -> bool {
        let tmin = (self.min - r.ro) * r.inv_dir;
        let tmax = (self.max - r.ro) * r.inv_dir;
        let t1 = tmin.fast_min(tmax);
        let t2 = tmin.fast_max(tmax);

        let [x1, y1, z1, _] = t1.to_array();
        let [x2, y2, z2, _] = t2.to_array();

        let tnear = ray_t.min.max(x1.max(y1.max(z1)));
        let tfar = ray_t.max.min(x2.min(y2.min(z2)));

        tnear <= tfar
    }

    const fn axis_interval(&self, i: usize) -> Interval {
        match i {
            0 => self.x,
            1 => self.y,
            2 => self.z,
            _ => unimplemented!(),
        }
    }

    const fn longest_axis(&self) -> usize {
        if self.x.size() > self.y.size() {
            if self.x.size() > self.z.size() {
                0
            } else {
                2
            }
        } else if self.y.size() > self.z.size() {
            1
        } else {
            2
        }
    }

    const fn pad_to_minimum(&mut self) {
        let delta = 0.0001;
        if self.x.size() < delta {
            self.x = self.x.expand(delta);
        }
        if self.y.size() < delta {
            self.y = self.y.expand(delta);
        }
        if self.z.size() < delta {
            self.z = self.z.expand(delta);
        }

        self.min = wide::f64x4::new([self.x.min, self.y.min, self.z.min, 0.0]);
        self.max = wide::f64x4::new([self.x.max, self.y.max, self.z.max, 0.0]);
    }

    pub const fn expand(&self, delta: f64) -> AABBox {
        AABBox::new(
            self.x.expand(delta),
            self.y.expand(delta),
            self.z.expand(delta),
        )
    }
}

impl Add<V3> for AABBox {
    type Output = AABBox;

    fn add(self, rhs: V3) -> Self::Output {
        AABBox::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Add<AABBox> for V3 {
    type Output = AABBox;

    fn add(self, rhs: AABBox) -> Self::Output {
        rhs + self
    }
}

#[derive(Debug, Clone)]
pub enum BvhInner {
    Node(Box<BvhNode>),
    Leaf(Hittable),
}

impl BvhInner {
    pub fn hits(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        match self {
            Self::Node(n) => n.hits(r, ray_t),
            Self::Leaf(l) => l.hits(r, ray_t),
        }
    }
}

// There is definitely a break even point in terms of the number of number of hittables
// in the scene and the utility of the bvh_tree in terms of the overhead from checking
// hits against the bounding boxes.
// It's probably worth defining a heuristic to check against the resulting tree to see
// if it is worthwhile using it or not.
#[derive(Debug, Clone)]
pub struct BvhNode {
    pub left: BvhInner,
    pub right: BvhInner,
    pub bbox: AABBox,
}

impl BvhNode {
    pub fn new_from_hittables(objects: Vec<Hittable>) -> BvhNode {
        Self::new(objects)
    }

    pub fn new(mut objects: Vec<Hittable>) -> BvhNode {
        let mut bbox = AABBox::EMPTY;
        for obj in objects.iter() {
            bbox = AABBox::new_enclosing(bbox, obj.bounding_box());
        }

        let (left, right) = match objects.len() {
            1 => (
                BvhInner::Leaf(objects.remove(0)),
                BvhInner::Leaf(Hittable::Empty),
            ),
            2 => (
                BvhInner::Leaf(objects.remove(0)),
                BvhInner::Leaf(objects.remove(0)),
            ),
            _ => {
                let axis = bbox.longest_axis();
                objects.sort_by(|a, b| {
                    let a_axis_interval = a.bounding_box().axis_interval(axis);
                    let b_axis_interval = b.bounding_box().axis_interval(axis);

                    a_axis_interval.min.total_cmp(&b_axis_interval.min)
                });

                let right = BvhNode::new(objects.split_off(objects.len() / 2));
                let left = BvhNode::new(objects);

                (
                    BvhInner::Node(Box::new(left)),
                    BvhInner::Node(Box::new(right)),
                )
            }
        };

        Self { left, right, bbox }
    }

    pub fn max_depth(&self) -> usize {
        match (&self.left, &self.right) {
            (BvhInner::Leaf(_), BvhInner::Leaf(_)) => 1,
            (BvhInner::Node(n), BvhInner::Leaf(_)) => n.max_depth() + 1,
            (BvhInner::Leaf(_), BvhInner::Node(m)) => m.max_depth() + 1,
            (BvhInner::Node(n), BvhInner::Node(m)) => n.max_depth().max(m.max_depth()) + 1,
        }
    }

    pub fn hits(&self, r: &Ray, mut ray_t: Interval) -> Option<HitRecord> {
        if !self.bbox.hits(r, ray_t) {
            return None;
        }

        let hit_left = match self.left.hits(r, ray_t) {
            Some(h) => {
                ray_t = Interval::new(ray_t.min, h.t);
                Some(h)
            }
            None => None,
        };

        self.right.hits(r, ray_t).or(hit_left)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_test_case::test_case;

    fn bbox(x1: f64, x2: f64, y1: f64, y2: f64, z1: f64, z2: f64) -> AABBox {
        AABBox::new(
            Interval::new(x1, x2),
            Interval::new(y1, y2),
            Interval::new(z1, z2),
        )
    }

    #[test_case(
        bbox(1.1, 2.1, 1.2, 2.2, 1.3, 2.3),
        bbox(1.1, 2.1, 1.2, 2.2, 1.3, 2.3),
        bbox(1.1, 2.1, 1.2, 2.2, 1.3, 2.3);
        "idempotent"
    )]
    #[test_case(
        bbox(1.1, 3.0, 1.2, 3.0, 1.3, 3.0),
        bbox(2.0, 5.1, 2.0, 5.2, 2.0, 5.3),
        bbox(1.1, 5.1, 1.2, 5.2, 1.3, 5.3);
        "overlapping"
    )]
    #[test_case(
        bbox(1.1, 2.0, 1.2, 2.0, 1.3, 2.0),
        bbox(3.0, 5.1, 3.0, 5.2, 3.0, 5.3),
        bbox(1.1, 5.1, 1.2, 5.2, 1.3, 5.3);
        "disjoint"
    )]
    #[test]
    fn enclosing_works(a: AABBox, b: AABBox, expected: AABBox) {
        let res = AABBox::new_enclosing(a, b);

        assert_eq!(res, expected);
    }
}
