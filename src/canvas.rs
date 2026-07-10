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
            crate::ui_components::paint_checkerboard(
                window,
                visible_canvas,
                layer_origin,
                24.0,
                rgb(0xcccccc),
                rgb(0xaaaaaa),
            );
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

            let tile_coords: Vec<crate::tile::TileCoords> = if is_active_layer_and_stroke {
                let mut coords_set = std::collections::HashSet::new();
                for &coords in raster.tiles.tiles.keys() {
                    coords_set.insert(coords);
                }
                if let Some(stroke) = &self.tool_state.read(cx).active_stroke {
                    for &coords in stroke.composited_tiles.keys() {
                        coords_set.insert(coords);
                    }
                }
                coords_set.into_iter().collect()
            } else {
                raster.tiles.tiles.keys().copied().collect()
            };

            for coords in tile_coords {
                let tx = coords.x;
                let ty = coords.y;

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
                    if img.is_none() {
                        layer_entity.update(cx, |layer, cx| {
                            img = layer.resolve_texture(coords, cx, &layer_entity.downgrade());
                        });
                    }
                    img
                } else {
                    layer_entity.update(cx, |layer, cx| {
                        layer.resolve_texture(coords, cx, &layer_entity.downgrade())
                    })
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
