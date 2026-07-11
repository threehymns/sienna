use crate::blend::{BlendMode, composite_pixel};

#[test]
fn test_normal_blend() {
    let fg = (0, 0, 255, 255); // Red (c2=255)
    let bg = (0, 255, 0, 255); // Green (c1=255)
    let result = composite_pixel(bg, fg, BlendMode::Normal, 1.0);
    assert_eq!(result, (0, 0, 255, 255)); // Expected: Red
}

#[test]
fn test_multiply_blend() {
    let fg = (128, 128, 128, 255); // 50% gray
    let bg = (255, 0, 0, 255); // Blue (c0=255)
    let result = composite_pixel(bg, fg, BlendMode::Multiply, 1.0);
    assert_eq!(result, (128, 0, 0, 255));
}

#[test]
fn test_alpha_compositing() {
    let fg = (0, 0, 255, 255); // Red
    let bg = (0, 0, 0, 255); // Black
    let result = composite_pixel(bg, fg, BlendMode::Normal, 0.5);

    assert_eq!(result.3, 255);
    assert_eq!(result.2, 128); // 0.5 * 255
    assert_eq!(result.0, 0);
    assert_eq!(result.1, 0);
}

#[test]
fn test_darken_blend() {
    let fg = (100, 200, 50, 255);
    let bg = (150, 150, 150, 255);
    let result = composite_pixel(bg, fg, BlendMode::Darken, 1.0);
    assert_eq!(result, (100, 150, 50, 255)); // min of channels
}

#[test]
fn test_color_burn_blend() {
    let fg = (128, 128, 128, 255);
    let bg = (192, 192, 192, 255);
    let result = composite_pixel(bg, fg, BlendMode::ColorBurn, 1.0);
    assert_eq!(result, (129, 129, 129, 255));
}

#[test]
fn test_screen_blend() {
    let fg = (128, 128, 128, 255);
    let bg = (128, 128, 128, 255);
    let result = composite_pixel(bg, fg, BlendMode::Screen, 1.0);
    assert_eq!(result, (192, 192, 192, 255));
}

#[test]
fn test_lighten_blend() {
    let fg = (100, 200, 50, 255);
    let bg = (150, 150, 150, 255);
    let result = composite_pixel(bg, fg, BlendMode::Lighten, 1.0);
    assert_eq!(result, (150, 200, 150, 255)); // max of channels
}

#[test]
fn test_color_dodge_blend() {
    let fg = (128, 128, 128, 255);
    let bg = (64, 64, 64, 255);
    let result = composite_pixel(bg, fg, BlendMode::ColorDodge, 1.0);
    assert_eq!(result, (129, 129, 129, 255));
}

#[test]
fn test_overlay_blend() {
    let fg = (192, 192, 192, 255);
    let bg = (128, 128, 128, 255);
    let result = composite_pixel(bg, fg, BlendMode::Overlay, 1.0);
    assert_eq!(result, (192, 192, 192, 255));
}

#[test]
fn test_soft_light_blend() {
    let fg = (64, 64, 64, 255); // cs = 0.25
    let bg = (128, 128, 128, 255); // cb = 0.5
    // cs <= 0.5: 0.5 - (1.0 - 2*0.25)*0.5*(1.0-0.5) = 0.5 - 0.5*0.5*0.5 = 0.5 - 0.125 = 0.375 -> 96
    let result = composite_pixel(bg, fg, BlendMode::SoftLight, 1.0);
    assert_eq!(result, (96, 96, 96, 255));
}

#[test]
fn test_hard_light_blend() {
    let fg = (64, 64, 64, 255); // cs = 0.25
    let bg = (128, 128, 128, 255); // cb = 0.5
    // cs <= 0.5: 2.0 * 0.5 * 0.25 = 0.25 -> 64
    let result = composite_pixel(bg, fg, BlendMode::HardLight, 1.0);
    assert_eq!(result, (64, 64, 64, 255));
}

#[test]
fn test_difference_blend() {
    let fg = (200, 100, 50, 255);
    let bg = (50, 200, 100, 255);
    // |50 - 200| = 150
    // |200 - 100| = 100
    // |100 - 50| = 50
    let result = composite_pixel(bg, fg, BlendMode::Difference, 1.0);
    assert_eq!(result, (150, 100, 50, 255));
}
