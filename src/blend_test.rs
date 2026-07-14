use crate::blend::{BgraTuple, BlendMode, composite_pixel};

#[track_caller]
fn assert_blend(bg: BgraTuple, fg: BgraTuple, mode: BlendMode, expected: BgraTuple, opacity: f32) {
    let result = composite_pixel(bg, fg, mode, opacity);
    assert_eq!(result, expected);
}

#[test]
fn test_normal_blend() {
    let fg = (0, 0, 255, 255); // Red
    let bg = (0, 255, 0, 255); // Green
    assert_blend(bg, fg, BlendMode::Normal, (0, 0, 255, 255), 1.0);
}

#[test]
fn test_multiply_blend() {
    let fg = (128, 128, 128, 255); // 50% gray
    let bg = (255, 0, 0, 255); // Blue
    assert_blend(bg, fg, BlendMode::Multiply, (128, 0, 0, 255), 1.0);
}

#[test]
fn test_alpha_compositing() {
    let fg = (0, 0, 255, 255); // Red
    let bg = (0, 0, 0, 255); // Black
    assert_blend(bg, fg, BlendMode::Normal, (0, 0, 128, 255), 0.5);
}

#[test]
fn test_darken_blend() {
    let fg = (100, 200, 50, 255);
    let bg = (150, 150, 150, 255);
    assert_blend(bg, fg, BlendMode::Darken, (100, 150, 50, 255), 1.0);
}

#[test]
fn test_color_burn_blend() {
    let fg = (128, 128, 128, 255);
    let bg = (192, 192, 192, 255);
    assert_blend(bg, fg, BlendMode::ColorBurn, (129, 129, 129, 255), 1.0);
}

#[test]
fn test_screen_blend() {
    let fg = (128, 128, 128, 255);
    let bg = (128, 128, 128, 255);
    assert_blend(bg, fg, BlendMode::Screen, (192, 192, 192, 255), 1.0);
}

#[test]
fn test_lighten_blend() {
    let fg = (100, 200, 50, 255);
    let bg = (150, 150, 150, 255);
    assert_blend(bg, fg, BlendMode::Lighten, (150, 200, 150, 255), 1.0);
}

#[test]
fn test_color_dodge_blend() {
    let fg = (128, 128, 128, 255);
    let bg = (64, 64, 64, 255);
    assert_blend(bg, fg, BlendMode::ColorDodge, (129, 129, 129, 255), 1.0);
}

#[test]
fn test_overlay_blend() {
    let fg = (192, 192, 192, 255);
    let bg = (128, 128, 128, 255);
    assert_blend(bg, fg, BlendMode::Overlay, (192, 192, 192, 255), 1.0);
}

#[test]
fn test_soft_light_blend() {
    let fg = (64, 64, 64, 255);
    let bg = (128, 128, 128, 255);
    assert_blend(bg, fg, BlendMode::SoftLight, (96, 96, 96, 255), 1.0);
}

#[test]
fn test_hard_light_blend() {
    let fg = (64, 64, 64, 255);
    let bg = (128, 128, 128, 255);
    assert_blend(bg, fg, BlendMode::HardLight, (64, 64, 64, 255), 1.0);
}

#[test]
fn test_difference_blend() {
    let fg = (200, 100, 50, 255);
    let bg = (50, 200, 100, 255);
    assert_blend(bg, fg, BlendMode::Difference, (150, 100, 50, 255), 1.0);
}
