use crate::document::{Document, Layer};
use crate::tool::ToolState;
use gpui::*;
use std::sync::Arc;

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

        let (doc_size, transform) = {
            let doc = self.document.read(cx);
            (doc.size, doc.transform)
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
        let doc = self.document.read(cx);
        let tool_state = self.tool_state.read(cx);
        let active_layer_index = doc.active_layer_index;

        for (layer_idx, layer_entity) in doc.layers.iter().enumerate() {
            let layer = layer_entity.read(cx);
            let Layer::Raster(raster) = layer;
            if !raster.visible {
                continue;
            }

            // If this is the active layer and a stroke is in progress,
            // render the stroke buffer's composited image instead of the layer cache.
            let render_image = if layer_idx == active_layer_index {
                tool_state
                    .active_stroke
                    .as_ref()
                    .and_then(|s| s.stroke_buffer.render_image.as_ref())
            } else {
                None
            };

            if let Some(render_image) = render_image {
                let _ = window.paint_image(
                    layer_bounds,
                    Corners::default(),
                    render_image.clone(),
                    0,
                    false,
                );
                continue;
            }

            // Normal layer rendering from cache
            if let Some(render_image) = &raster.render_cache {
                let render_image: Arc<RenderImage> = render_image.clone();
                let _ =
                    window.paint_image(layer_bounds, Corners::default(), render_image, 0, false);
            }
        }

        // Brush cursor
        let is_brush_or_eraser = tool_state.active_tool == crate::tool::Tool::Brush
            || tool_state.active_tool == crate::tool::Tool::Eraser;

        if is_brush_or_eraser {
            let mouse_pos = window.mouse_position();
            if bounds.contains(&mouse_pos) {
                let brush_size = tool_state.brush_size * transform.scale;
                let half = brush_size / 2.0;
                let cursor_bounds = Bounds {
                    origin: mouse_pos
                        - Point {
                            x: px(half),
                            y: px(half),
                        },
                    size: Size {
                        width: px(brush_size),
                        height: px(brush_size),
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
