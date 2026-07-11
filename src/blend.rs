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

pub fn blend_normal(_cb: f32, cs: f32) -> f32 {
    cs
}
pub fn blend_multiply(cb: f32, cs: f32) -> f32 {
    cb * cs
}
pub fn blend_darken(cb: f32, cs: f32) -> f32 {
    cb.min(cs)
}
pub fn blend_color_burn(cb: f32, cs: f32) -> f32 {
    if cb >= 1.0 {
        1.0
    } else if cs <= 0.0 {
        0.0
    } else {
        (1.0 - (1.0 - cb) / cs).max(0.0)
    }
}
pub fn blend_screen(cb: f32, cs: f32) -> f32 {
    cb + cs - cb * cs
}
pub fn blend_lighten(cb: f32, cs: f32) -> f32 {
    cb.max(cs)
}
pub fn blend_color_dodge(cb: f32, cs: f32) -> f32 {
    if cb <= 0.0 {
        0.0
    } else if cs >= 1.0 {
        1.0
    } else {
        (cb / (1.0 - cs)).min(1.0)
    }
}
pub fn blend_overlay(cb: f32, cs: f32) -> f32 {
    if cb <= 0.5 {
        2.0 * cb * cs
    } else {
        1.0 - 2.0 * (1.0 - cb) * (1.0 - cs)
    }
}
pub fn blend_soft_light(cb: f32, cs: f32) -> f32 {
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
pub fn blend_hard_light(cb: f32, cs: f32) -> f32 {
    if cs <= 0.5 {
        2.0 * cb * cs
    } else {
        1.0 - 2.0 * (1.0 - cb) * (1.0 - cs)
    }
}
pub fn blend_difference(cb: f32, cs: f32) -> f32 {
    (cb - cs).abs()
}

fn get_blend_fn(mode: BlendMode) -> fn(f32, f32) -> f32 {
    match mode {
        BlendMode::Normal => blend_normal,
        BlendMode::Multiply => blend_multiply,
        BlendMode::Darken => blend_darken,
        BlendMode::ColorBurn => blend_color_burn,
        BlendMode::Screen => blend_screen,
        BlendMode::Lighten => blend_lighten,
        BlendMode::ColorDodge => blend_color_dodge,
        BlendMode::Overlay => blend_overlay,
        BlendMode::SoftLight => blend_soft_light,
        BlendMode::HardLight => blend_hard_light,
        BlendMode::Difference => blend_difference,
    }
}

fn composite_channel(
    cb: f32,
    cs: f32,
    bg_a: f32,
    fg_a: f32,
    out_a: f32,
    blend_fn: fn(f32, f32) -> f32,
) -> f32 {
    ((1.0 - bg_a) * fg_a * cs + bg_a * (1.0 - fg_a) * cb + fg_a * bg_a * blend_fn(cb, cs)) / out_a
}

pub fn composite_pixel(
    bg: (u8, u8, u8, u8),
    fg: (u8, u8, u8, u8),
    mode: BlendMode,
    fg_opacity: f32,
) -> (u8, u8, u8, u8) {
    let fg_a = (fg.3 as f32 / 255.0) * fg_opacity;
    let bg_a = bg.3 as f32 / 255.0;

    let out_a = fg_a + bg_a * (1.0 - fg_a);
    if out_a <= 0.0 {
        return (0, 0, 0, 0);
    }

    let bg_c0 = bg.0 as f32 / 255.0;
    let bg_c1 = bg.1 as f32 / 255.0;
    let bg_c2 = bg.2 as f32 / 255.0;

    let fg_c0 = fg.0 as f32 / 255.0;
    let fg_c1 = fg.1 as f32 / 255.0;
    let fg_c2 = fg.2 as f32 / 255.0;

    let blend_fn = get_blend_fn(mode);

    let out_c0 = composite_channel(bg_c0, fg_c0, bg_a, fg_a, out_a, blend_fn);
    let out_c1 = composite_channel(bg_c1, fg_c1, bg_a, fg_a, out_a, blend_fn);
    let out_c2 = composite_channel(bg_c2, fg_c2, bg_a, fg_a, out_a, blend_fn);

    (
        (out_c0.clamp(0.0, 1.0) * 255.0).round() as u8,
        (out_c1.clamp(0.0, 1.0) * 255.0).round() as u8,
        (out_c2.clamp(0.0, 1.0) * 255.0).round() as u8,
        (out_a.clamp(0.0, 1.0) * 255.0).round() as u8,
    )
}
