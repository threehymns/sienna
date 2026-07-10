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
    /// The stroke-only pixels (BGRA).
    pub tiles: crate::tile::TileGrid,
    pub width: u32,
    pub height: u32,
    /// Bounding box of all pixels modified during this stroke.
    pub dirty_rect: Option<DirtyRect>,
    /// Snapshot of the layer pixels at stroke start.
    pub layer_snapshot: crate::tile::TileGrid,
    /// The composited result.
    pub composited: crate::tile::TileGrid,
    /// Cached RenderImage for the compositor output.
    pub render_image: Option<Arc<RenderImage>>,
    /// Whether the composited image needs rebuilding.
    pub needs_composite: bool,
    /// Whether this is an eraser stroke
    pub is_eraser: bool,
    pub dirty_tiles: std::collections::HashSet<(u32, u32)>,
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
    pub fn new(
        width: u32,
        height: u32,
        layer_snapshot: crate::tile::TileGrid,
        is_eraser: bool,
    ) -> Self {
        let composited = layer_snapshot.clone();
        Self {
            tiles: crate::tile::TileGrid::new(width, height),
            width,
            height,
            dirty_rect: None,
            layer_snapshot,
            composited,
            render_image: None,
            needs_composite: false,
            is_eraser,
            dirty_tiles: std::collections::HashSet::new(),
        }
    }

    /// Apply a single dab to the stroke buffer.
    pub fn apply_dab(&mut self, dab: &DabParams, is_eraser: bool) {
        let dab_rect = DirtyRect::from_dab(dab.position, dab.size, self.width, self.height);
        self.dirty_rect = Some(match self.dirty_rect {
            Some(existing) => existing.union(&dab_rect),
            None => dab_rect,
        });

        let tx_start = dab_rect.x / crate::tile::TILE_SIZE;
        let ty_start = dab_rect.y / crate::tile::TILE_SIZE;
        let tx_end = (dab_rect.x + dab_rect.w).div_ceil(crate::tile::TILE_SIZE);
        let ty_end = (dab_rect.y + dab_rect.h).div_ceil(crate::tile::TILE_SIZE);
        for ty in ty_start..ty_end {
            for tx in tx_start..tx_end {
                self.dirty_tiles.insert((tx, ty));
            }
        }

        if is_eraser {
            BrushEngine::erase_dab(
                &mut self.tiles,
                dab.position,
                dab.size,
                dab.hardness,
                dab.opacity * dab.flow,
            );
        } else {
            BrushEngine::draw_dab(
                &mut self.tiles,
                dab.position,
                dab.color,
                dab.size,
                dab.hardness,
                dab.opacity * dab.flow,
            );
        }
        self.needs_composite = true;
    }

    /// Composite the stroke buffer over the layer snapshot, but only within the dirty tiles.
    pub fn composite_dirty(&mut self) -> std::collections::HashSet<(u32, u32)> {
        if !self.needs_composite {
            return std::collections::HashSet::new();
        }
        let dirty = std::mem::take(&mut self.dirty_tiles);
        for &(tx, ty) in &dirty {
            if self.tiles.tiles.contains_key(&(tx, ty)) {
                let stroke_tile = self.tiles.tiles.get(&(tx, ty)).unwrap();
                let snapshot_tile = self.layer_snapshot.tiles.get(&(tx, ty));
                let mut comp_tile = crate::tile::Tile::new();
                comp_tile.composite_stroke(stroke_tile, snapshot_tile, self.is_eraser);
                self.composited.tiles.insert((tx, ty), comp_tile);
            }
        }
        self.needs_composite = false;
        self.dirty_rect = None;
        dirty
    }

    /// Build a RenderImage from the composited pixels.
    #[allow(dead_code)]
    pub fn build_render_image(&mut self) -> Arc<RenderImage> {
        let buffer =
            image::RgbaImage::from_raw(self.width, self.height, self.composited.to_monolithic())
                .unwrap();
        let frame = image::Frame::new(buffer);
        let img = Arc::new(RenderImage::new(smallvec::smallvec![frame]));
        self.render_image = Some(img.clone());
        img
    }

    /// Finalize: return the composited pixels as the new layer state.
    pub fn finalize(mut self) -> (crate::tile::TileGrid, crate::tile::TileGrid) {
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
        layer_snapshot: crate::tile::TileGrid,
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
    /// Returns the coordinates of any dirty tiles that were updated.
    pub fn feed(&mut self, raw_pos: Point<f32>) -> std::collections::HashSet<(u32, u32)> {
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
                return std::collections::HashSet::new();
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
            self.stroke_buffer.composite_dirty()
        } else {
            std::collections::HashSet::new()
        }
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

    /// Finalize the stroke, returning (before_tiles, after_tiles) for undo.
    pub fn finalize(self) -> (crate::tile::TileGrid, crate::tile::TileGrid) {
        self.stroke_buffer.finalize()
    }
}
