use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Bgra {
    pub b: u8,
    pub g: u8,
    pub r: u8,
    pub a: u8,
}

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

pub fn blend_normal(_backdrop: f32, source: f32) -> f32 {
    source
}
pub fn blend_multiply(backdrop: f32, source: f32) -> f32 {
    backdrop * source
}
pub fn blend_darken(backdrop: f32, source: f32) -> f32 {
    backdrop.min(source)
}
pub fn blend_color_burn(backdrop: f32, source: f32) -> f32 {
    if backdrop >= 1.0 {
        1.0
    } else if source <= 0.0 {
        0.0
    } else {
        (1.0 - (1.0 - backdrop) / source).max(0.0)
    }
}
pub fn blend_screen(backdrop: f32, source: f32) -> f32 {
    backdrop + source - backdrop * source
}
pub fn blend_lighten(backdrop: f32, source: f32) -> f32 {
    backdrop.max(source)
}
pub fn blend_color_dodge(backdrop: f32, source: f32) -> f32 {
    if backdrop <= 0.0 {
        0.0
    } else if source >= 1.0 {
        1.0
    } else {
        (backdrop / (1.0 - source)).min(1.0)
    }
}
pub fn blend_overlay(backdrop: f32, source: f32) -> f32 {
    if backdrop <= 0.5 {
        2.0 * backdrop * source
    } else {
        1.0 - 2.0 * (1.0 - backdrop) * (1.0 - source)
    }
}
pub fn blend_soft_light(backdrop: f32, source: f32) -> f32 {
    if source <= 0.5 {
        backdrop - (1.0 - 2.0 * source) * backdrop * (1.0 - backdrop)
    } else {
        let d = if backdrop <= 0.25 {
            ((16.0 * backdrop - 12.0) * backdrop + 4.0) * backdrop
        } else {
            backdrop.sqrt()
        };
        backdrop + (2.0 * source - 1.0) * (d - backdrop)
    }
}
pub fn blend_hard_light(backdrop: f32, source: f32) -> f32 {
    blend_overlay(source, backdrop)
}
pub fn blend_difference(backdrop: f32, source: f32) -> f32 {
    (backdrop - source).abs()
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
    backdrop: f32,
    source: f32,
    bg_a: f32,
    fg_a: f32,
    out_a: f32,
    blend_fn: fn(f32, f32) -> f32,
) -> f32 {
    ((1.0 - bg_a) * fg_a * source
        + bg_a * (1.0 - fg_a) * backdrop
        + fg_a * bg_a * blend_fn(backdrop, source))
        / out_a
}

pub fn composite_pixel(bg: Bgra, fg: Bgra, mode: BlendMode, fg_opacity: f32) -> Bgra {
    let fg_a = (fg.a as f32 / 255.0) * fg_opacity;
    let bg_a = bg.a as f32 / 255.0;

    let out_a = fg_a + bg_a * (1.0 - fg_a);
    if out_a <= 0.0 {
        return Bgra {
            b: 0,
            g: 0,
            r: 0,
            a: 0,
        };
    }

    let bg_b = bg.b as f32 / 255.0;
    let bg_g = bg.g as f32 / 255.0;
    let bg_r = bg.r as f32 / 255.0;

    let fg_b = fg.b as f32 / 255.0;
    let fg_g = fg.g as f32 / 255.0;
    let fg_r = fg.r as f32 / 255.0;

    let blend_fn = get_blend_fn(mode);

    let out_b = composite_channel(bg_b, fg_b, bg_a, fg_a, out_a, blend_fn);
    let out_g = composite_channel(bg_g, fg_g, bg_a, fg_a, out_a, blend_fn);
    let out_r = composite_channel(bg_r, fg_r, bg_a, fg_a, out_a, blend_fn);

    Bgra {
        b: (out_b.clamp(0.0, 1.0) * 255.0).round() as u8,
        g: (out_g.clamp(0.0, 1.0) * 255.0).round() as u8,
        r: (out_r.clamp(0.0, 1.0) * 255.0).round() as u8,
        a: (out_a.clamp(0.0, 1.0) * 255.0).round() as u8,
    }
}
