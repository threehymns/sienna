use crate::document::{Document, Layer};
use crate::tool::ToolState;
use gpui::*;

use gpui_component::ActiveTheme;

pub struct CanvasElement {
    document: Entity<Document>,
    tool_state: Entity<ToolState>,
}

impl CanvasElement {
    pub fn new(document: Entity<Document>, tool_state: Entity<ToolState>) -> Self {
        Self {
            document,
            tool_state,
        }
    }
}

pub struct CanvasLayoutState {
    #[allow(dead_code)]
    layout_id: LayoutId,
}

pub struct CanvasPrepaintState {
    _hitbox: Hitbox,
}

impl Element for CanvasElement {
    type RequestLayoutState = CanvasLayoutState;
    type PrepaintState = CanvasPrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let layout_id = window.request_layout(
            Style {
                size: Size {
                    width: relative(1.0).into(),
                    height: relative(1.0).into(),
                },
                ..Default::default()
            },
            vec![],
            cx,
        );
        (layout_id, CanvasLayoutState { layout_id })
    }

    fn prepaint(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
        let hitbox = window.insert_hitbox(bounds, HitboxBehavior::Normal);
        CanvasPrepaintState { _hitbox: hitbox }
    }

    fn paint(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        // Dark background
        window.paint_quad(fill(bounds, rgb(0x1a1a1a)));

        let (doc_size, transform, layers) = {
            let doc = self.document.read(cx);
            (doc.size, doc.transform, doc.layers.clone())
        };

        let layer_origin = bounds.origin + transform.offset.map(px);
        let layer_size = doc_size.map(|v| px(v as f32 * transform.scale));
        let layer_bounds = Bounds {
            origin: layer_origin,
            size: layer_size,
        };

        // Checkerboard background (transparency indicator)
        let visible_canvas = bounds.intersect(&layer_bounds);
        if visible_canvas.size.width > px(0.0) && visible_canvas.size.height > px(0.0) {
            window.paint_quad(fill(visible_canvas, rgb(0xcccccc)));

            // Use a fixed screen-space checkerboard size to avoid grid density scaling up when zoomed out.
            let check_size = 24.0;
            let check_px = px(check_size);

            let rel_left = visible_canvas.origin.x - layer_origin.x;
            let rel_top = visible_canvas.origin.y - layer_origin.y;
            let rel_right = rel_left + visible_canvas.size.width;
            let rel_bottom = rel_top + visible_canvas.size.height;

            let first_col = (rel_left / check_px).floor() as i32;
            let first_row = (rel_top / check_px).floor() as i32;
            let last_col = (rel_right / check_px).ceil() as i32;
            let last_row = (rel_bottom / check_px).ceil() as i32;

            for row in first_row..last_row {
                for col in first_col..last_col {
                    if (row + col) % 2 != 0 {
                        let check_bounds = Bounds {
                            origin: Point {
                                x: layer_origin.x + px(col as f32 * check_size),
                                y: layer_origin.y + px(row as f32 * check_size),
                            },
                            size: Size {
                                width: check_px,
                                height: check_px,
                            },
                        };
                        let clipped = check_bounds.intersect(&visible_canvas);
                        if clipped.size.width > px(0.0) && clipped.size.height > px(0.0) {
                            window.paint_quad(fill(clipped, rgb(0xaaaaaa)));
                        }
                    }
                }
            }
        }

        // Render layers
        let active_layer_index = self.document.read(cx).active_layer_index;
        let (active_tool, brush_size, has_active_stroke) = {
            let ts = self.tool_state.read(cx);
            (ts.active_tool, ts.brush_size, ts.active_stroke.is_some())
        };

        for (layer_idx, layer_entity) in layers.iter().enumerate() {
            let layer = layer_entity.read(cx);
            let Layer::Raster(raster) = layer;
            if !raster.visible {
                continue;
            }

            let is_active_layer_and_stroke = layer_idx == active_layer_index && has_active_stroke;

            let tile_coords: Vec<(u32, u32)> = if is_active_layer_and_stroke {
                self.tool_state
                    .read(cx)
                    .active_stroke
                    .as_ref()
                    .unwrap()
                    .composited_tiles
                    .keys()
                    .copied()
                    .collect()
            } else {
                raster.tiles.tiles.keys().copied().collect()
            };

            for coords in tile_coords {
                let tx = coords.0;
                let ty = coords.1;

                let tile_origin = layer_origin
                    + Point {
                        x: px(tx as f32 * crate::tile::TILE_SIZE as f32 * transform.scale),
                        y: px(ty as f32 * crate::tile::TILE_SIZE as f32 * transform.scale),
                    };
                let tile_size = Size {
                    width: px(crate::tile::TILE_SIZE as f32 * transform.scale),
                    height: px(crate::tile::TILE_SIZE as f32 * transform.scale),
                };
                let tile_bounds = Bounds {
                    origin: tile_origin,
                    size: tile_size,
                };

                let clipped = tile_bounds.intersect(&visible_canvas);
                if clipped.size.width <= px(0.0) || clipped.size.height <= px(0.0) {
                    continue;
                }

                let render_image = if is_active_layer_and_stroke {
                    let mut img = None;
                    self.tool_state.update(cx, |ts, _cx| {
                        if let Some(stroke) = &ts.active_stroke {
                            img = stroke.render_cache.get(&coords).cloned();
                        }
                    });
                    img
                } else {
                    let mut img = None;
                    layer_entity.update(cx, |layer, _cx| {
                        let Layer::Raster(raster) = layer;
                        let entry = raster.render_cache.entry(coords).or_insert_with(|| {
                            let tile = raster.tiles.tiles.get(&coords).unwrap();
                            tile.build_render_image()
                        });
                        img = Some(entry.clone());
                    });
                    img
                };

                if let Some(render_image) = render_image {
                    let _ =
                        window.paint_image(tile_bounds, Corners::default(), render_image, 0, false);
                }
            }
        }

        // Brush cursor
        let is_brush_or_eraser =
            active_tool == crate::tool::Tool::Brush || active_tool == crate::tool::Tool::Eraser;

        if is_brush_or_eraser {
            let mouse_pos = window.mouse_position();
            if bounds.contains(&mouse_pos) {
                let brush_size_scaled = brush_size * transform.scale;
                let half = brush_size_scaled / 2.0;
                let cursor_bounds = Bounds {
                    origin: mouse_pos
                        - Point {
                            x: px(half),
                            y: px(half),
                        },
                    size: Size {
                        width: px(brush_size_scaled),
                        height: px(brush_size_scaled),
                    },
                };
                // Draw a circular cursor outline
                window.paint_quad(fill(cursor_bounds, cx.theme().foreground.alpha(0.15)));
            }
        }
    }
}

impl IntoElement for CanvasElement {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}
