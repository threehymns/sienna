use crate::blend::{Bgra, BlendMode, composite_pixel};

#[test]
fn test_normal_blend() {
    let fg = Bgra {
        b: 0,
        g: 0,
        r: 255,
        a: 255,
    }; // Red
    let bg = Bgra {
        b: 0,
        g: 255,
        r: 0,
        a: 255,
    }; // Green
    let result = composite_pixel(bg, fg, BlendMode::Normal, 1.0);
    assert_eq!(
        result,
        Bgra {
            b: 0,
            g: 0,
            r: 255,
            a: 255
        }
    ); // Expected: Red
}

#[test]
fn test_multiply_blend() {
    let fg = Bgra {
        b: 128,
        g: 128,
        r: 128,
        a: 255,
    }; // 50% gray
    let bg = Bgra {
        b: 255,
        g: 0,
        r: 0,
        a: 255,
    }; // Blue (b=255)
    let result = composite_pixel(bg, fg, BlendMode::Multiply, 1.0);
    assert_eq!(
        result,
        Bgra {
            b: 128,
            g: 0,
            r: 0,
            a: 255
        }
    );
}

#[test]
fn test_alpha_compositing() {
    let fg = Bgra {
        b: 0,
        g: 0,
        r: 255,
        a: 255,
    }; // Red
    let bg = Bgra {
        b: 0,
        g: 0,
        r: 0,
        a: 255,
    }; // Black
    let result = composite_pixel(bg, fg, BlendMode::Normal, 0.5);

    assert_eq!(result.a, 255);
    assert_eq!(result.r, 128); // 0.5 * 255
    assert_eq!(result.b, 0);
    assert_eq!(result.g, 0);
}

#[test]
fn test_darken_blend() {
    let fg = Bgra {
        b: 100,
        g: 200,
        r: 50,
        a: 255,
    };
    let bg = Bgra {
        b: 150,
        g: 150,
        r: 150,
        a: 255,
    };
    let result = composite_pixel(bg, fg, BlendMode::Darken, 1.0);
    assert_eq!(
        result,
        Bgra {
            b: 100,
            g: 150,
            r: 50,
            a: 255
        }
    ); // min of channels
}

#[test]
fn test_color_burn_blend() {
    let fg = Bgra {
        b: 128,
        g: 128,
        r: 128,
        a: 255,
    };
    let bg = Bgra {
        b: 192,
        g: 192,
        r: 192,
        a: 255,
    };
    let result = composite_pixel(bg, fg, BlendMode::ColorBurn, 1.0);
    assert_eq!(
        result,
        Bgra {
            b: 129,
            g: 129,
            r: 129,
            a: 255
        }
    );
}

#[test]
fn test_screen_blend() {
    let fg = Bgra {
        b: 128,
        g: 128,
        r: 128,
        a: 255,
    };
    let bg = Bgra {
        b: 128,
        g: 128,
        r: 128,
        a: 255,
    };
    let result = composite_pixel(bg, fg, BlendMode::Screen, 1.0);
    assert_eq!(
        result,
        Bgra {
            b: 192,
            g: 192,
            r: 192,
            a: 255
        }
    );
}

#[test]
fn test_lighten_blend() {
    let fg = Bgra {
        b: 100,
        g: 200,
        r: 50,
        a: 255,
    };
    let bg = Bgra {
        b: 150,
        g: 150,
        r: 150,
        a: 255,
    };
    let result = composite_pixel(bg, fg, BlendMode::Lighten, 1.0);
    assert_eq!(
        result,
        Bgra {
            b: 150,
            g: 200,
            r: 150,
            a: 255
        }
    ); // max of channels
}

#[test]
fn test_color_dodge_blend() {
    let fg = Bgra {
        b: 128,
        g: 128,
        r: 128,
        a: 255,
    };
    let bg = Bgra {
        b: 64,
        g: 64,
        r: 64,
        a: 255,
    };
    let result = composite_pixel(bg, fg, BlendMode::ColorDodge, 1.0);
    assert_eq!(
        result,
        Bgra {
            b: 129,
            g: 129,
            r: 129,
            a: 255
        }
    );
}

#[test]
fn test_overlay_blend() {
    let fg = Bgra {
        b: 192,
        g: 192,
        r: 192,
        a: 255,
    };
    let bg = Bgra {
        b: 128,
        g: 128,
        r: 128,
        a: 255,
    };
    let result = composite_pixel(bg, fg, BlendMode::Overlay, 1.0);
    assert_eq!(
        result,
        Bgra {
            b: 192,
            g: 192,
            r: 192,
            a: 255
        }
    );
}

#[test]
fn test_soft_light_blend() {
    let fg = Bgra {
        b: 64,
        g: 64,
        r: 64,
        a: 255,
    };
    let bg = Bgra {
        b: 128,
        g: 128,
        r: 128,
        a: 255,
    };
    let result = composite_pixel(bg, fg, BlendMode::SoftLight, 1.0);
    assert_eq!(
        result,
        Bgra {
            b: 96,
            g: 96,
            r: 96,
            a: 255
        }
    );
}

#[test]
fn test_hard_light_blend() {
    let fg = Bgra {
        b: 64,
        g: 64,
        r: 64,
        a: 255,
    };
    let bg = Bgra {
        b: 128,
        g: 128,
        r: 128,
        a: 255,
    };
    let result = composite_pixel(bg, fg, BlendMode::HardLight, 1.0);
    assert_eq!(
        result,
        Bgra {
            b: 64,
            g: 64,
            r: 64,
            a: 255
        }
    );
}

#[test]
fn test_difference_blend() {
    let fg = Bgra {
        b: 200,
        g: 100,
        r: 50,
        a: 255,
    };
    let bg = Bgra {
        b: 50,
        g: 200,
        r: 100,
        a: 255,
    };
    let result = composite_pixel(bg, fg, BlendMode::Difference, 1.0);
    assert_eq!(
        result,
        Bgra {
            b: 150,
            g: 100,
            r: 50,
            a: 255
        }
    );
}
