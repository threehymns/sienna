use gpui::*;

/// Stateless brush engine — provides optimized dab rendering functions.
/// All methods are static and operate directly on pixel buffers.
/// Pixels are expected in BGRA format.
pub struct BrushEngine;

impl BrushEngine {
    /// Draw a brush dab at `pos` into the pixel buffer.
    /// Uses a max-alpha compositing model within a single stroke:
    /// each dab's contribution is max'd with existing stroke alpha,
    /// preventing darkening on overlapping dabs (like Photoshop's flow behavior).
    #[inline]
    pub fn draw_dab(
        pixels: &mut [u8],
        width: u32,
        height: u32,
        pos: Point<f32>,
        color: Rgba,
        size: f32,
        hardness: f32,
        strength: f32, // opacity * flow combined
    ) {
        let radius = size / 2.0;
        let radius_sq = radius * radius;
        let falloff_start = radius * hardness;
        let falloff_start_sq = falloff_start * falloff_start;
        let falloff_range_inv = 1.0 / (radius - falloff_start).max(0.001);

        // Pre-clamp to canvas bounds — no per-pixel bounds checks
        let x_start = (pos.x - radius).floor().max(0.0) as u32;
        let y_start = (pos.y - radius).floor().max(0.0) as u32;
        let x_end = (pos.x + radius).ceil().min(width as f32) as u32;
        let y_end = (pos.y + radius).ceil().min(height as f32) as u32;

        let b_src = (color.b * 255.0) as u8;
        let g_src = (color.g * 255.0) as u8;
        let r_src = (color.r * 255.0) as u8;

        for y in y_start..y_end {
            let row_base = (y * width) as usize * 4;
            let dy = y as f32 + 0.5 - pos.y;
            let dy_sq = dy * dy;

            for x in x_start..x_end {
                let dx = x as f32 + 0.5 - pos.x;
                let dist_sq = dx * dx + dy_sq;

                if dist_sq > radius_sq {
                    continue;
                }

                let dist = dist_sq.sqrt();

                // Hardness falloff
                let intensity = if dist_sq <= falloff_start_sq {
                    1.0
                } else {
                    let t = (dist - falloff_start) * falloff_range_inv;
                    (1.0 - t).max(0.0)
                };

                // Sub-pixel edge anti-aliasing
                let edge_aa = (radius - dist).min(1.0).max(0.0);
                let dab_alpha = strength * intensity * edge_aa;

                if dab_alpha <= 0.0 {
                    continue;
                }

                let idx = row_base + (x as usize) * 4;

                // Max-alpha compositing for within-stroke buildup:
                // The stroke buffer accumulates max(existing, new) alpha,
                // which prevents over-darkening when dabs overlap.
                // Color is written with the max alpha.
                let existing_a = pixels[idx + 3] as f32 / 255.0;
                let new_a = dab_alpha.min(1.0);

                if new_a > existing_a {
                    pixels[idx] = b_src;
                    pixels[idx + 1] = g_src;
                    pixels[idx + 2] = r_src;
                    pixels[idx + 3] = (new_a * 255.0) as u8;
                }
                // If existing alpha is already higher, this dab doesn't contribute.
                // This is the "flow" model: repeated passes in the same area
                // don't exceed the brush opacity ceiling.
            }
        }
    }

    /// Erase dab: writes alpha into the stroke buffer that will be used
    /// to reduce the layer alpha during compositing.
    /// For the eraser, the stroke buffer's alpha represents "erase strength" —
    /// higher alpha = more erasure.
    #[inline]
    pub fn erase_dab(
        pixels: &mut [u8],
        width: u32,
        height: u32,
        pos: Point<f32>,
        size: f32,
        hardness: f32,
        strength: f32,
    ) {
        let radius = size / 2.0;
        let radius_sq = radius * radius;
        let falloff_start = radius * hardness;
        let falloff_start_sq = falloff_start * falloff_start;
        let falloff_range_inv = 1.0 / (radius - falloff_start).max(0.001);

        let x_start = (pos.x - radius).floor().max(0.0) as u32;
        let y_start = (pos.y - radius).floor().max(0.0) as u32;
        let x_end = (pos.x + radius).ceil().min(width as f32) as u32;
        let y_end = (pos.y + radius).ceil().min(height as f32) as u32;

        for y in y_start..y_end {
            let row_base = (y * width) as usize * 4;
            let dy = y as f32 + 0.5 - pos.y;
            let dy_sq = dy * dy;

            for x in x_start..x_end {
                let dx = x as f32 + 0.5 - pos.x;
                let dist_sq = dx * dx + dy_sq;

                if dist_sq > radius_sq {
                    continue;
                }

                let dist = dist_sq.sqrt();
                let intensity = if dist_sq <= falloff_start_sq {
                    1.0
                } else {
                    let t = (dist - falloff_start) * falloff_range_inv;
                    (1.0 - t).max(0.0)
                };

                let edge_aa = (radius - dist).min(1.0).max(0.0);
                let erase_alpha = strength * intensity * edge_aa;

                if erase_alpha <= 0.0 {
                    continue;
                }

                let idx = row_base + (x as usize) * 4;
                // Max-alpha accumulation for eraser too
                let existing = pixels[idx + 3] as f32 / 255.0;
                let new_val = erase_alpha.min(1.0);
                if new_val > existing {
                    // Store erase intensity as alpha; RGB channels are unused for eraser
                    pixels[idx + 3] = (new_val * 255.0) as u8;
                }
            }
        }
    }
}
