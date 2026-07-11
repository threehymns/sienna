use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BlendMode {
    #[default]
    Normal,
    Multiply,
    Darken,
    ColorBurn,
    Screen,
    Lighten,
    ColorDodge,
    Overlay,
    SoftLight,
    HardLight,
    Difference,
}

pub fn composite_pixel(src: [u8; 4], bg: [u8; 4], mode: BlendMode, src_opacity: f32) -> [u8; 4] {
    let src_a = (src[3] as f32 / 255.0) * src_opacity;
    let bg_a = bg[3] as f32 / 255.0;

    let out_a = src_a + bg_a * (1.0 - src_a);
    if out_a <= 0.0 {
        return [0, 0, 0, 0];
    }

    let src_b = src[0] as f32 / 255.0;
    let src_g = src[1] as f32 / 255.0;
    let src_r = src[2] as f32 / 255.0;

    let bg_b = bg[0] as f32 / 255.0;
    let bg_g = bg[1] as f32 / 255.0;
    let bg_r = bg[2] as f32 / 255.0;

    let blend = |cb: f32, cs: f32| -> f32 {
        match mode {
            BlendMode::Normal => cs,
            BlendMode::Multiply => cb * cs,
            BlendMode::Darken => cb.min(cs),
            BlendMode::ColorBurn => {
                if cb >= 1.0 {
                    1.0
                } else if cs <= 0.0 {
                    0.0
                } else {
                    (1.0 - (1.0 - cb) / cs).max(0.0)
                }
            }
            BlendMode::Screen => cb + cs - cb * cs,
            BlendMode::Lighten => cb.max(cs),
            BlendMode::ColorDodge => {
                if cb <= 0.0 {
                    0.0
                } else if cs >= 1.0 {
                    1.0
                } else {
                    (cb / (1.0 - cs)).min(1.0)
                }
            }
            BlendMode::Overlay => {
                if cb <= 0.5 {
                    2.0 * cb * cs
                } else {
                    1.0 - 2.0 * (1.0 - cb) * (1.0 - cs)
                }
            }
            BlendMode::SoftLight => {
                if cs <= 0.5 {
                    cb - (1.0 - 2.0 * cs) * cb * (1.0 - cb)
                } else {
                    let d = if cb <= 0.25 {
                        ((16.0 * cb - 12.0) * cb + 4.0) * cb
                    } else {
                        cb.sqrt()
                    };
                    cb + (2.0 * cs - 1.0) * (d - cb)
                }
            }
            BlendMode::HardLight => {
                if cs <= 0.5 {
                    2.0 * cb * cs
                } else {
                    1.0 - 2.0 * (1.0 - cb) * (1.0 - cs)
                }
            }
            BlendMode::Difference => (cb - cs).abs(),
        }
    };

    let out_b = ((1.0 - bg_a) * src_a * src_b
        + bg_a * (1.0 - src_a) * bg_b
        + src_a * bg_a * blend(bg_b, src_b))
        / out_a;
    let out_g = ((1.0 - bg_a) * src_a * src_g
        + bg_a * (1.0 - src_a) * bg_g
        + src_a * bg_a * blend(bg_g, src_g))
        / out_a;
    let out_r = ((1.0 - bg_a) * src_a * src_r
        + bg_a * (1.0 - src_a) * bg_r
        + src_a * bg_a * blend(bg_r, src_r))
        / out_a;

    [
        (out_b.clamp(0.0, 1.0) * 255.0).round() as u8,
        (out_g.clamp(0.0, 1.0) * 255.0).round() as u8,
        (out_r.clamp(0.0, 1.0) * 255.0).round() as u8,
        (out_a.clamp(0.0, 1.0) * 255.0).round() as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_blend() {
        // Full opacity red over full opacity green
        let src = [0, 0, 255, 255]; // BGRA
        let bg = [0, 255, 0, 255];
        let result = composite_pixel(src, bg, BlendMode::Normal, 1.0);
        assert_eq!(result, [0, 0, 255, 255]); // Expected: red
    }

    #[test]
    fn test_multiply_blend() {
        let src = [128, 128, 128, 255]; // 50% gray
        let bg = [255, 0, 0, 255]; // Pure blue (BGRA, B=255)
        let result = composite_pixel(src, bg, BlendMode::Multiply, 1.0);
        // Multiply 1.0 * 0.5 = 0.5 (128)
        assert_eq!(result, [128, 0, 0, 255]);
    }

    #[test]
    fn test_alpha_compositing() {
        // Semi-transparent red over black background
        let src = [0, 0, 255, 255]; // Pure red
        let bg = [0, 0, 0, 255]; // Black
        let result = composite_pixel(src, bg, BlendMode::Normal, 0.5); // 50% opacity

        // Output should be ~50% red, 100% alpha
        assert_eq!(result[3], 255);
        assert_eq!(result[2], 128); // 0.5 * 255 = 127.5 rounds to 128
        assert_eq!(result[0], 0);
        assert_eq!(result[1], 0);
    }
}
