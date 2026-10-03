use crate::{
    v3::{self, Onb},
    Ray, Rng, P3, V3,
};
use std::cmp::max;

pub trait Camera: Send + Sync {
    fn get_ray(&self, sample: V3, r: &mut Ray, rng: &mut Rng);
    fn basis(&self) -> Onb;
    fn image_dims(&self) -> (usize, usize);
    fn viewport_dims(&self) -> (f32, f32);
    fn viewport_top_left(&self) -> V3;
}

#[derive(Debug, Copy, Clone)]
pub struct SimpleCamera {
    image_w: usize,
    image_h: usize,
    viewport_w: f32,
    viewport_h: f32,
    viewport_top_left: V3,
    basis: Onb,
    center: P3,         // camera center
    defocus_angle: f32, // angle of the defocus disk
    defocus_disk_u: V3, // defocus disk horizontal radius
    defocus_disk_v: V3, // defocus disk vertical radius
}

impl SimpleCamera {
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        aspect_ratio: f32,
        image_w: usize,
        fov: f32,
        look_from: P3,
        look_at: P3,
        v_up: V3,
        defocus_angle: f32,
        focus_dist: f32,
    ) -> Self {
        let image_h = max(1, (image_w as f32 / aspect_ratio) as usize);

        // viewport dimensions
        let theta = fov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_h = 2.0 * h * focus_dist;
        let viewport_w = viewport_h * (image_w as f32 / image_h as f32);

        // Calculate the u,v,w unit basis vectors for the camera coordinate frame.
        let w = (look_from - look_at).normalize();
        let u = v_up.cross(w).normalize();
        let v = w.cross(u);

        // Calculate the camera defocus disk basis vectors.
        let defocus_radius = focus_dist * (defocus_angle / 2.0).to_radians().tan();
        let defocus_disk_u = u * defocus_radius;
        let defocus_disk_v = v * defocus_radius;

        let viewport_u = viewport_w * u;
        let viewport_v = viewport_h * -v;
        let viewport_top_left = look_from - (focus_dist * w) - viewport_u / 2.0 - viewport_v / 2.0;

        Self {
            image_w,
            image_h,
            viewport_w,
            viewport_h,
            basis: Onb { u, v, w },
            center: look_from,
            viewport_top_left,
            defocus_angle,
            defocus_disk_u,
            defocus_disk_v,
        }
    }

    // Returns a random point in the camera defocus disk.
    fn defocus_disk_sample(&self, rng: &mut Rng) -> P3 {
        let p = v3::random_in_unit_disk(rng);

        self.center + (p.x * self.defocus_disk_u) + (p.y * self.defocus_disk_v)
    }
}

impl Camera for SimpleCamera {
    /// Construct a camera ray originating from the defocus disk and directed at a randomly
    /// sampled point around the pixel location i, j.
    fn get_ray(&self, sample: V3, r: &mut Ray, rng: &mut Rng) {
        let ray_origin = if self.defocus_angle <= 0.0 {
            self.center
        } else {
            self.defocus_disk_sample(rng)
        };

        r.set(ray_origin, sample - ray_origin);
    }

    fn basis(&self) -> Onb {
        self.basis
    }

    fn viewport_top_left(&self) -> V3 {
        self.viewport_top_left
    }

    fn image_dims(&self) -> (usize, usize) {
        (self.image_w, self.image_h)
    }

    fn viewport_dims(&self) -> (f32, f32) {
        (self.viewport_w, self.viewport_h)
    }
}
