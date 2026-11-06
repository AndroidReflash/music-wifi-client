use crate::settings::ThemeColor;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsv {
    pub h: f32,
    pub s: f32,
    pub v: f32,
}

impl Hsv {
    pub fn new(h: f32, s: f32, v: f32) -> Self {
        Self {
            h: h.rem_euclid(360.0),
            s: s.clamp(0.0, 1.0),
            v: v.clamp(0.0, 1.0),
        }
    }

    pub fn from_theme(color: ThemeColor) -> Self {
        rgb_to_hsv(color.r, color.g, color.b)
    }

    pub fn to_theme(self) -> ThemeColor {
        let (r, g, b) = hsv_to_rgb(self.h, self.s, self.v);
        ThemeColor { r, g, b, a: 255 }
    }
}

pub fn rgb_to_hsv(r: u8, g: u8, b: u8) -> Hsv {
    let rf = r as f32 / 255.0;
    let gf = g as f32 / 255.0;
    let bf = b as f32 / 255.0;
    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let delta = max - min;

    let h = if delta <= f32::EPSILON {
        0.0
    } else if (max - rf).abs() <= f32::EPSILON {
        60.0 * (((gf - bf) / delta) % 6.0)
    } else if (max - gf).abs() <= f32::EPSILON {
        60.0 * (((bf - rf) / delta) + 2.0)
    } else {
        60.0 * (((rf - gf) / delta) + 4.0)
    };

    let s = if max <= f32::EPSILON { 0.0 } else { delta / max };
    let v = max;

    Hsv::new(h, s, v)
}

pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let h = h.rem_euclid(360.0);
    let s = s.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);

    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (rf, gf, bf) = match (h / 60.0).floor() as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    let to_u8 = |value: f32| ((value + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    (to_u8(rf), to_u8(gf), to_u8(bf))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_primary_colors() {
        for (r, g, b) in [(255, 0, 0), (0, 255, 0), (0, 0, 255), (0, 255, 255), (180, 130, 255)] {
            let hsv = rgb_to_hsv(r, g, b);
            let (rr, gg, bb) = hsv_to_rgb(hsv.h, hsv.s, hsv.v);
            assert!((rr as i16 - r as i16).abs() <= 1);
            assert!((gg as i16 - g as i16).abs() <= 1);
            assert!((bb as i16 - b as i16).abs() <= 1);
        }
    }

    #[test]
    fn black_and_white() {
        let black = rgb_to_hsv(0, 0, 0);
        assert_eq!(hsv_to_rgb(black.h, black.s, black.v), (0, 0, 0));
        let white = rgb_to_hsv(255, 255, 255);
        assert_eq!(hsv_to_rgb(white.h, white.s, white.v), (255, 255, 255));
    }
}
