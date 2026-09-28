use crate::{
    color, leak_ptr,
    material::Material,
    texture::{SolidColor, Texture},
    Color, HitRecord, Ray, Rng,
};

#[derive(Debug, Clone)]
pub struct DiffuseLight {
    texture: &'static dyn Texture,
}

impl DiffuseLight {
    pub fn new_mat(texture: &'static dyn Texture) -> &'static dyn Material {
        leak_ptr!(Self { texture })
    }

    pub fn new_color(albedo: impl Into<Color>) -> &'static dyn Material {
        Self::new_mat(leak_ptr!(SolidColor::new(albedo)))
    }
}

impl Material for DiffuseLight {
    fn needs_uv_calc(&self) -> bool {
        self.texture.needs_uv_calc()
    }

    fn color_emitted(&self, hr: &HitRecord) -> Color {
        if hr.front_face {
            self.texture.value(hr.u, hr.v, hr.p)
        } else {
            color::BLACK
        }
    }

    fn scattering_pdf(&self, _r_in: &Ray, _r_out: &Ray, _hr: &HitRecord) -> f32 {
        0.0
    }

    fn scatter(
        &self,
        _r_in: &Ray,
        _r_out: &mut Ray,
        _hr: &HitRecord,
        _pdf: &mut f32,
        _rng: &mut Rng,
    ) -> Option<Color> {
        None
    }
}
