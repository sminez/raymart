use crate::v3::V3;

pub type Color = V3;

impl Color {
    pub fn print_ppm(&self) {
        // TODO: really this should be a different type with checks on construction
        debug_assert!(
            self.x >= 0.0 && self.x <= 1.0,
            "self.x out of bounds {}",
            self.x
        );
        debug_assert!(
            self.y >= 0.0 && self.y <= 1.0,
            "self.y out of bounds {}",
            self.y
        );
        debug_assert!(
            self.z >= 0.0 && self.z <= 1.0,
            "self.z out of bounds {}",
            self.z
        );

        // Translate the [0,1] component values to the byte range [0,255].
        let ir = (255.999 * self.x) as i64;
        let ig = (255.999 * self.y) as i64;
        let ib = (255.999 * self.z) as i64;

        debug_assert!(ir >= 0 && ir <= 255, "ir out of bounds {ir}");
        debug_assert!(ig >= 0 && ig <= 255, "ig out of bounds {ig}");
        debug_assert!(ib >= 0 && ib <= 255, "ib out of bounds {ib}");

        println!("{ir} {ig} {ib}");
    }
}
