//! Axis-aligned bounding boxes and Bounding Volume Hierarchies
//! See Section 3 of https://raytracing.github.io/books/RayTracingTheNextWeek.html for the details

use crate::{
    hit::{HitRecord, Hittable, Interval},
    Ray, P3, V3,
};
use std::ops::Add;

pub const MAX_BVH_DEPTH: usize = 16;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct AABBox {
    pub x: Interval,
    pub y: Interval,
    pub z: Interval,
    min: wide::f32x4,
    max: wide::f32x4,
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
            min: wide::f32x4::ZERO,
            max: wide::f32x4::ZERO,
        };
        bbox.pad_to_minimum();

        bbox
    }

    pub const fn new_enclosing(a: AABBox, b: AABBox) -> AABBox {
        let mut bbox = AABBox {
            x: Interval::new_enclosing(a.x, b.x),
            y: Interval::new_enclosing(a.y, b.y),
            z: Interval::new_enclosing(a.z, b.z),
            min: wide::f32x4::ZERO,
            max: wide::f32x4::ZERO,
        };
        bbox.pad_to_minimum();

        bbox
    }

    fn new_containing(hittables: &[Hittable]) -> Self {
        let mut bbox = AABBox::EMPTY;
        for obj in hittables.iter() {
            bbox = AABBox::new_enclosing(bbox, obj.bounding_box());
        }

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
            min: wide::f32x4::ZERO,
            max: wide::f32x4::ZERO,
        };
        bbox.pad_to_minimum();

        bbox
    }

    pub fn hit_dist(&self, r: &Ray, ray_t: Interval) -> f32 {
        let tmin = (self.min - r.ro) * r.inv_dir;
        let tmax = (self.max - r.ro) * r.inv_dir;
        let t1 = tmin.fast_min(tmax);
        let t2 = tmin.fast_max(tmax);

        let [x1, y1, z1, _] = t1.to_array();
        let [x2, y2, z2, _] = t2.to_array();

        let tnear = ray_t.min.max(x1.max(y1.max(z1)));
        let tfar = ray_t.max.min(x2.min(y2.min(z2)));

        let hit = tfar >= tnear && tfar > 0.0;
        if hit {
            tnear.max(0.0)
        } else {
            f32::INFINITY
        }
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

        self.min = wide::f32x4::new([self.x.min, self.y.min, self.z.min, 0.0]);
        self.max = wide::f32x4::new([self.x.max, self.y.max, self.z.max, 0.0]);
    }

    pub const fn expand(&self, delta: f32) -> AABBox {
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
pub struct FatNode {
    bbox: AABBox,
    start: usize, // start of children if n is None, else start of hittables
    n: Option<usize>,
}

impl FatNode {
    fn new(bbox: AABBox, start: usize) -> Self {
        Self {
            bbox,
            start,
            n: None,
        }
    }
}

fn split(
    parent_idx: usize,
    start: usize,
    n: usize,
    depth: usize,
    nodes: &mut Vec<FatNode>,
    hittables: &mut [Hittable],
) {
    if n == 1 || depth >= MAX_BVH_DEPTH {
        // remaining hittables sit in this node
        let parent = &mut nodes[parent_idx];
        parent.start = start;
        parent.n = Some(n);
        return;
    }

    // Split into two halves and recursively split the children
    let axis = nodes[parent_idx].bbox.longest_axis();
    hittables[start..(start + n)].sort_by(|a, b| {
        let a_axis_interval = a.bounding_box().axis_interval(axis);
        let b_axis_interval = b.bounding_box().axis_interval(axis);
        a_axis_interval.min.total_cmp(&b_axis_interval.min)
    });

    let nleft = n / 2;
    let nright = n - nleft;

    let lbbox = AABBox::new_containing(&hittables[start..start + nleft]);
    nodes.push(FatNode::new(lbbox, start));
    let rbbox = AABBox::new_containing(&hittables[start + nleft..start + n]);
    nodes.push(FatNode::new(rbbox, start + nleft));

    let lidx = nodes.len() - 2;
    let ridx = nodes.len() - 1;
    nodes[parent_idx].start = lidx;

    split(lidx, start, nleft, depth + 1, nodes, hittables);
    split(ridx, start + nleft, nright, depth + 1, nodes, hittables);
}

#[derive(Debug, Clone)]
pub struct Node {
    min: wide::f32x4,
    max: wide::f32x4,
    start: usize, // start of children if n is None, else start of hittables
    n: Option<usize>,
}

impl Node {
    #[inline]
    pub fn hit_dist(&self, r: &Ray, ray_t: Interval) -> f32 {
        let tmin = (self.min - r.ro) * r.inv_dir;
        let tmax = (self.max - r.ro) * r.inv_dir;
        let t1 = tmin.fast_min(tmax);
        let t2 = tmin.fast_max(tmax);

        let [x1, y1, z1, _] = t1.to_array();
        let [x2, y2, z2, _] = t2.to_array();

        let tnear = ray_t.min.max(x1.max(y1.max(z1)));
        let tfar = ray_t.max.min(x2.min(y2.min(z2)));

        let hit = tfar >= tnear && tfar > 0.0;
        if hit {
            tnear.max(0.0)
        } else {
            f32::INFINITY
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Bvh {
    hittables: Vec<Hittable>,
    nodes: Vec<Node>,
    pub bbox: AABBox,
}

impl Bvh {
    pub fn new(mut hittables: Vec<Hittable>) -> Self {
        let bbox = AABBox::new_containing(&hittables);
        let mut fat_nodes = vec![FatNode::new(bbox, 0)];

        split(0, 0, hittables.len(), 0, &mut fat_nodes, &mut hittables);
        let nodes = fat_nodes
            .into_iter()
            .map(|n| Node {
                min: n.bbox.min,
                max: n.bbox.max,
                start: n.start,
                n: n.n,
            })
            .collect();

        Self {
            hittables,
            nodes,
            bbox,
        }
    }

    pub fn hits(
        &self,
        r: &Ray,
        mut ray_t: Interval,
        stack: &mut [usize; MAX_BVH_DEPTH],
    ) -> Option<HitRecord> {
        let mut hr = None;
        let mut i = 1;
        stack[0] = 0;

        while i > 0 {
            i -= 1;
            let node = &self.nodes[stack[i]];

            if let Some(n) = node.n {
                // leaf node: check for hits
                for leaf in &self.hittables[node.start..node.start + n] {
                    if let Some(rec) = leaf.hits(r, ray_t) {
                        ray_t.max = rec.t;
                        hr = Some(rec);
                    }
                }
            } else {
                // check bbox for left and right children and push them to the stack
                // if they intersect the ray
                let left = &self.nodes[node.start];
                let right = &self.nodes[node.start + 1];
                let ldist = left.hit_dist(r, ray_t);
                let rdist = right.hit_dist(r, ray_t);

                let ((a, adist), (b, bdist)) = if ldist < rdist {
                    ((node.start, ldist), (node.start + 1, rdist))
                } else {
                    ((node.start + 1, rdist), (node.start, ldist))
                };

                if adist < ray_t.max {
                    stack[i] = a;
                    i += 1;
                }
                if bdist < ray_t.max {
                    stack[i] = b;
                    i += 1;
                }
            }
        }

        hr
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_test_case::test_case;

    fn bbox(x1: f32, x2: f32, y1: f32, y2: f32, z1: f32, z2: f32) -> AABBox {
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
