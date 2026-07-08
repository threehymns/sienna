use crate::brush::BrushEngine;
use gpui::*;
use std::sync::Arc;

/// Represents a single dab to be rendered.
#[derive(Clone, Copy, Debug)]
pub struct DabParams {
    pub position: Point<f32>,
    pub size: f32,
    pub color: Rgba,
    pub opacity: f32,
    pub flow: f32,
    pub hardness: f32,
}

/// The active stroke's pixel buffer with dirty-rect tracking.
/// During a stroke, dabs are composited into this buffer.
/// On display, this buffer is alpha-blended over the layer snapshot.
pub struct StrokeBuffer {
    /// The stroke-only pixels (BGRA, premultiplied-ish for compositing).
    /// Same dimensions as the document layer.
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
    /// Bounding box of all pixels modified during this stroke.
    /// Used for incremental GPU upload and minimal compositing.
    pub dirty_rect: Option<DirtyRect>,
    /// Snapshot of the layer pixels at stroke start (for undo and compositing).
    pub layer_snapshot: Vec<u8>,
    /// The composited result: layer_snapshot + stroke pixels.
    /// This is what gets turned into a RenderImage for display.
    pub composited: Vec<u8>,
    /// Cached RenderImage for the compositor output.
    pub render_image: Option<Arc<RenderImage>>,
    /// Whether the composited image needs rebuilding.
    pub needs_composite: bool,
    /// Whether this is an eraser stroke
    pub is_eraser: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct DirtyRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl DirtyRect {
    pub fn from_dab(pos: Point<f32>, size: f32, canvas_w: u32, canvas_h: u32) -> Self {
        let radius = size / 2.0;
        let x_start = (pos.x - radius).floor().max(0.0) as u32;
        let y_start = (pos.y - radius).floor().max(0.0) as u32;
        let x_end = (pos.x + radius).ceil().min(canvas_w as f32) as u32;
        let y_end = (pos.y + radius).ceil().min(canvas_h as f32) as u32;
        Self {
            x: x_start,
            y: y_start,
            w: x_end.saturating_sub(x_start),
            h: y_end.saturating_sub(y_start),
        }
    }

    pub fn union(&self, other: &DirtyRect) -> DirtyRect {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let right = (self.x + self.w).max(other.x + other.w);
        let bottom = (self.y + self.h).max(other.y + other.h);
        DirtyRect {
            x,
            y,
            w: right - x,
            h: bottom - y,
        }
    }
}

impl StrokeBuffer {
    pub fn new(width: u32, height: u32, layer_snapshot: Vec<u8>, is_eraser: bool) -> Self {
        let pixel_count = (width * height * 4) as usize;
        let composited = layer_snapshot.clone();
        Self {
            pixels: vec![0u8; pixel_count],
            width,
            height,
            dirty_rect: None,
            layer_snapshot,
            composited,
            render_image: None,
            needs_composite: false,
            is_eraser,
        }
    }

    /// Apply a single dab to the stroke buffer.
    pub fn apply_dab(&mut self, dab: &DabParams, is_eraser: bool) {
        let dab_rect = DirtyRect::from_dab(dab.position, dab.size, self.width, self.height);
        self.dirty_rect = Some(match self.dirty_rect {
            Some(existing) => existing.union(&dab_rect),
            None => dab_rect,
        });

        if is_eraser {
            BrushEngine::erase_dab(
                &mut self.pixels,
                self.width,
                self.height,
                dab.position,
                dab.size,
                dab.hardness,
                dab.opacity * dab.flow,
            );
        } else {
            BrushEngine::draw_dab(
                &mut self.pixels,
                self.width,
                self.height,
                dab.position,
                dab.color,
                dab.size,
                dab.hardness,
                dab.opacity * dab.flow,
            );
        }
        self.needs_composite = true;
    }

    /// Composite the stroke buffer over the layer snapshot, but only within the dirty rect.
    /// This produces the final pixel data for display.
    pub fn composite_dirty(&mut self) {
        if !self.needs_composite {
            return;
        }
        let Some(rect) = self.dirty_rect else { return };

        let w = self.width;
        let x_end = (rect.x + rect.w).min(w);
        let y_end = (rect.y + rect.h).min(self.height);

        for y in rect.y..y_end {
            let row_offset = (y * w) as usize * 4;
            for x in rect.x..x_end {
                let idx = row_offset + (x as usize) * 4;

                let stroke_a = self.pixels[idx + 3] as f32 / 255.0;
                if stroke_a <= 0.0 {
                    // No stroke contribution in this pixel.
                    // Important: if we are over an area that WAS dirty but now is 0 (unlikely with max-alpha),
                    // we should restore from snapshot.
                    // However, with max-alpha, pixels only go from 0 to something.
                    continue;
                }

                if self.is_eraser {
                    // Eraser: reduce the alpha of the existing layer
                    let bg_b = self.layer_snapshot[idx];
                    let bg_g = self.layer_snapshot[idx + 1];
                    let bg_r = self.layer_snapshot[idx + 2];
                    let bg_a = self.layer_snapshot[idx + 3] as f32 / 255.0;

                    let new_a = (bg_a * (1.0 - stroke_a)).max(0.0);

                    self.composited[idx] = bg_b;
                    self.composited[idx + 1] = bg_g;
                    self.composited[idx + 2] = bg_r;
                    self.composited[idx + 3] = (new_a * 255.0) as u8;
                } else {
                    // Brush: normal alpha blend (A over B)
                    let bg_b = self.layer_snapshot[idx] as f32;
                    let bg_g = self.layer_snapshot[idx + 1] as f32;
                    let bg_r = self.layer_snapshot[idx + 2] as f32;
                    let bg_a = self.layer_snapshot[idx + 3] as f32 / 255.0;

                    let fg_b = self.pixels[idx] as f32;
                    let fg_g = self.pixels[idx + 1] as f32;
                    let fg_r = self.pixels[idx + 2] as f32;

                    let one_minus_fg_a = 1.0 - stroke_a;
                    let out_a = stroke_a + bg_a * one_minus_fg_a;

                    if out_a > 0.0 {
                        let inv_out_a = 1.0 / out_a;
                        let bg_a_blend = bg_a * one_minus_fg_a;
                        self.composited[idx] =
                            ((fg_b * stroke_a + bg_b * bg_a_blend) * inv_out_a) as u8;
                        self.composited[idx + 1] =
                            ((fg_g * stroke_a + bg_g * bg_a_blend) * inv_out_a) as u8;
                        self.composited[idx + 2] =
                            ((fg_r * stroke_a + bg_r * bg_a_blend) * inv_out_a) as u8;
                        self.composited[idx + 3] = (out_a * 255.0) as u8;
                    } else {
                        self.composited[idx] = 0;
                        self.composited[idx + 1] = 0;
                        self.composited[idx + 2] = 0;
                        self.composited[idx + 3] = 0;
                    }
                }
            }
        }
        self.needs_composite = false;
        self.dirty_rect = None;
    }

    /// Build a RenderImage from the composited pixels.
    pub fn build_render_image(&mut self) -> Arc<RenderImage> {
        let buffer =
            image::RgbaImage::from_raw(self.width, self.height, self.composited.clone()).unwrap();
        let frame = image::Frame::new(buffer);
        let img = Arc::new(RenderImage::new(smallvec::smallvec![frame]));
        self.render_image = Some(img.clone());
        img
    }

    /// Finalize: return the composited pixels as the new layer state.
    pub fn finalize(mut self) -> (Vec<u8>, Vec<u8>) {
        self.composite_dirty();
        let after = self.composited;
        let before = self.layer_snapshot;
        (before, after)
    }
}

/// Handles input smoothing, dab spacing, and feeds dabs to StrokeBuffer.
pub struct StrokeAccumulator {
    /// Brush parameters for this stroke
    pub brush_size: f32,
    pub brush_opacity: f32,
    pub brush_flow: f32,
    pub brush_hardness: f32,
    pub brush_spacing: f32,
    pub brush_stabilization: f32,
    pub color: Rgba,
    pub is_eraser: bool,

    /// The last stabilized/smoothed position
    last_pos: Option<Point<f32>>,
    /// Distance accumulated since last dab
    dist_since_last_dab: f32,
    /// The active stroke buffer
    pub stroke_buffer: StrokeBuffer,
}

impl StrokeAccumulator {
    #[allow(clippy::too_many_arguments)]
    pub fn begin(
        width: u32,
        height: u32,
        layer_snapshot: Vec<u8>,
        brush_size: f32,
        brush_opacity: f32,
        brush_flow: f32,
        brush_hardness: f32,
        brush_spacing: f32,
        brush_stabilization: f32,
        color: Rgba,
        is_eraser: bool,
    ) -> Self {
        Self {
            brush_size,
            brush_opacity,
            brush_flow,
            brush_hardness,
            brush_spacing,
            brush_stabilization,
            color,
            is_eraser,
            last_pos: None,
            dist_since_last_dab: 0.0,
            stroke_buffer: StrokeBuffer::new(width, height, layer_snapshot, is_eraser),
        }
    }

    /// Feed a raw canvas-space position into the accumulator.
    /// Returns true if any dabs were placed (i.e., needs re-render).
    pub fn feed(&mut self, raw_pos: Point<f32>) -> bool {
        let smoothed = if let Some(last) = self.last_pos {
            let alpha = 1.0 - self.brush_stabilization;
            Point {
                x: last.x + (raw_pos.x - last.x) * alpha,
                y: last.y + (raw_pos.y - last.y) * alpha,
            }
        } else {
            raw_pos
        };

        let spacing_px = (self.brush_size * self.brush_spacing.max(0.05)).max(1.0);
        let mut placed_any = false;

        if let Some(last) = self.last_pos {
            let dx = smoothed.x - last.x;
            let dy = smoothed.y - last.y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < 0.5 {
                return false;
            }

            let dir_x = dx / dist;
            let dir_y = dy / dist;
            let mut cursor = spacing_px - self.dist_since_last_dab;

            while cursor <= dist {
                let p = Point {
                    x: last.x + dir_x * cursor,
                    y: last.y + dir_y * cursor,
                };
                self.place_dab(p);
                placed_any = true;
                cursor += spacing_px;
            }

            self.dist_since_last_dab = dist - (cursor - spacing_px);
        } else {
            // First point of the stroke — always place a dab
            self.place_dab(smoothed);
            placed_any = true;
            self.dist_since_last_dab = 0.0;
        }

        self.last_pos = Some(smoothed);

        if placed_any {
            // Composite dirty region for display
            self.stroke_buffer.composite_dirty();
        }

        placed_any
    }

    fn place_dab(&mut self, position: Point<f32>) {
        let dab = DabParams {
            position,
            size: self.brush_size,
            color: self.color,
            opacity: self.brush_opacity,
            flow: self.brush_flow,
            hardness: self.brush_hardness,
        };
        self.stroke_buffer.apply_dab(&dab, self.is_eraser);
    }

    /// Finalize the stroke, returning (before_pixels, after_pixels) for undo.
    pub fn finalize(self) -> (Vec<u8>, Vec<u8>) {
        self.stroke_buffer.finalize()
    }
}
