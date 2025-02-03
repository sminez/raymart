//! Axis-aligned bounding boxes and Bounding Volume Hierarchies
//! See Section 3 of https://raytracing.github.io/books/RayTracingTheNextWeek.html for the details

use crate::{
    hit::{Empty, HitRecord, Hittable, HittableList, Interval},
    Ray, P3,
};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct AABBox {
    x: Interval,
    y: Interval,
    z: Interval,
}

impl AABBox {
    pub const EMPTY: AABBox = AABBox::new(Interval::EMPTY, Interval::EMPTY, Interval::EMPTY);
    pub const UNIVERSE: AABBox =
        AABBox::new(Interval::UNIVERSE, Interval::UNIVERSE, Interval::UNIVERSE);

    pub const fn new(x: Interval, y: Interval, z: Interval) -> AABBox {
        AABBox { x, y, z }
    }

    pub const fn new_enclosing(a: AABBox, b: AABBox) -> AABBox {
        AABBox {
            x: Interval::new_enclosing(a.x, b.x),
            y: Interval::new_enclosing(a.y, b.y),
            z: Interval::new_enclosing(a.z, b.z),
        }
    }

    /// Treat the two points a and b as extrema for the bounding box, so we don't require a
    /// particular minimum/maximum coordinate order.
    pub fn new_from_points(a: P3, b: P3) -> AABBox {
        let (x1, x2) = if a.x <= b.x { (a.x, b.x) } else { (b.x, a.x) };
        let (y1, y2) = if a.y <= b.y { (a.y, b.y) } else { (b.y, a.y) };
        let (z1, z2) = if a.z <= b.z { (a.z, b.z) } else { (b.z, a.z) };

        AABBox {
            x: Interval::new(x1, x2),
            y: Interval::new(y1, y2),
            z: Interval::new(z1, z2),
        }
    }

    pub fn hits(&self, r: &Ray, mut ray_t: Interval) -> bool {
        for axis in 0..3 {
            let ax = self.axis_interval(axis);
            let adinv = 1.0 / r.dir[axis];

            let t0 = (ax.min - r.orig[axis]) * adinv;
            let t1 = (ax.max - r.orig[axis]) * adinv;

            if t0 < t1 {
                if t0 > ray_t.min {
                    ray_t.min = t0;
                }
                if t1 < ray_t.max {
                    ray_t.max = t1;
                }
            } else {
                if t1 > ray_t.min {
                    ray_t.min = t1;
                }
                if t0 < ray_t.max {
                    ray_t.max = t0;
                }
            }

            if ray_t.max <= ray_t.min {
                return false;
            }
        }

        true
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
}

#[derive(Debug, Clone, Copy)]
pub struct BvhNode {
    pub left: &'static dyn Hittable,
    pub right: &'static dyn Hittable,
    pub bbox: AABBox,
}

impl BvhNode {
    pub fn new_from_hittable_list(l: &HittableList) -> BvhNode {
        Self::new(l.objects.clone())
    }

    pub fn new(mut objects: Vec<&'static dyn Hittable>) -> BvhNode {
        let mut bbox = AABBox::EMPTY;
        for obj in objects.iter() {
            bbox = AABBox::new_enclosing(bbox, obj.bounding_box());
        }

        let (left, right) = match objects.len() {
            1 => (objects[0], Box::leak(Box::new(Empty)) as &dyn Hittable),
            2 => (objects[0], objects[1]),
            _ => {
                let axis = bbox.longest_axis();
                objects.sort_by(|a, b| {
                    let a_axis_interval = a.bounding_box().axis_interval(axis);
                    let b_axis_interval = b.bounding_box().axis_interval(axis);

                    a_axis_interval.min.total_cmp(&b_axis_interval.min)
                });

                let robjects = objects.split_off(objects.len() / 2);

                let left = BvhNode::new(objects);
                let right = BvhNode::new(robjects);

                (
                    Box::leak(Box::new(left)) as &dyn Hittable,
                    Box::leak(Box::new(right)) as &dyn Hittable,
                )
            }
        };

        let bbox = AABBox::new_enclosing(left.bounding_box(), right.bounding_box());

        Self { left, right, bbox }
    }
}

impl Hittable for BvhNode {
    fn hits(&self, r: &Ray, mut ray_t: Interval) -> Option<HitRecord> {
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

    fn bounding_box(&self) -> AABBox {
        self.bbox
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
