use crate::document::Layer;
use crate::tool::Tool;
use crate::ui_components::{icon_button, property_slider, tool_button};
use crate::workspace::Workspace;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::ActiveTheme;
use gpui_component::color_picker::ColorPicker;

#[derive(IntoElement)]
pub struct Toolbox {
    pub workspace: WeakEntity<Workspace>,
}

impl RenderOnce for Toolbox {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let workspace_entity = self.workspace.clone();
        let Some(workspace) = self.workspace.upgrade() else {
            return div();
        };
        let workspace = workspace.read(cx);
        let tool_state = workspace.tool_state.read(cx);
        let active_tool = tool_state.active_tool;
        let color_picker_state = workspace.color_picker_state.clone();

        div()
            .w(px(48.))
            .h_full()
            .bg(cx.theme().muted)
            .border_r(px(1.))
            .border_color(cx.theme().border)
            .flex()
            .flex_col()
            .items_center()
            .py(px(8.))
            .gap(px(8.))
            .child({
                let workspace_entity = workspace_entity.clone();
                tool_button(
                    "M",
                    "move-tool",
                    active_tool == Tool::Move,
                    move |_, _, cx| {
                        workspace_entity
                            .update(cx, |this, cx| {
                                this.tool_state.update(cx, |state, _cx| {
                                    state.active_tool = Tool::Move;
                                });
                            })
                            .ok();
                    },
                )
            })
            .child({
                let workspace_entity = workspace_entity.clone();
                tool_button(
                    "B",
                    "brush-tool",
                    active_tool == Tool::Brush,
                    move |_, _, cx| {
                        workspace_entity
                            .update(cx, |this, cx| {
                                this.tool_state.update(cx, |state, _cx| {
                                    state.active_tool = Tool::Brush;
                                });
                            })
                            .ok();
                    },
                )
            })
            .child({
                let workspace_entity = workspace_entity.clone();
                tool_button(
                    "E",
                    "eraser-tool",
                    active_tool == Tool::Eraser,
                    move |_, _, cx| {
                        workspace_entity
                            .update(cx, |this, cx| {
                                this.tool_state.update(cx, |state, _cx| {
                                    state.active_tool = Tool::Eraser;
                                });
                            })
                            .ok();
                    },
                )
            })
            .child({
                let workspace_entity = workspace_entity.clone();
                tool_button(
                    "P",
                    "picker-tool",
                    active_tool == Tool::ColorPicker,
                    move |_, _, cx| {
                        workspace_entity
                            .update(cx, |this, cx| {
                                this.tool_state.update(cx, |state, _cx| {
                                    state.active_tool = Tool::ColorPicker;
                                });
                            })
                            .ok();
                    },
                )
            })
            .child(div().w_full().h(px(1.)).bg(cx.theme().border))
            .child(
                div()
                    .id("sidebar-color-picker")
                    .child(ColorPicker::new(&color_picker_state)),
            )
    }
}

#[derive(IntoElement)]
pub struct PropertiesPanel {
    pub workspace: WeakEntity<Workspace>,
}

impl RenderOnce for PropertiesPanel {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let Some(workspace) = self.workspace.upgrade() else {
            return div();
        };
        let workspace = workspace.read(cx);
        let tool_state = workspace.tool_state.read(cx);
        let brush_size = tool_state.brush_size;
        let brush_opacity = tool_state.brush_opacity;
        let brush_flow = tool_state.brush_flow;
        let brush_hardness = tool_state.brush_hardness;
        let brush_spacing = tool_state.brush_spacing;
        let brush_stabilization = tool_state.brush_stabilization;

        div()
            .flex()
            .flex_col()
            .child(
                div()
                    .p(px(8.))
                    .border_b(px(1.))
                    .border_color(cx.theme().border)
                    .child("Properties"),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(12.))
                    .p(px(16.))
                    .child(property_slider(
                        "Size",
                        &workspace.brush_size_slider,
                        format!("{:.0}", brush_size),
                        cx,
                    ))
                    .child(property_slider(
                        "Opacity",
                        &workspace.brush_opacity_slider,
                        format!("{:.0}%", brush_opacity * 100.0),
                        cx,
                    ))
                    .child(property_slider(
                        "Flow",
                        &workspace.brush_flow_slider,
                        format!("{:.0}%", brush_flow * 100.0),
                        cx,
                    ))
                    .child(property_slider(
                        "Hardness",
                        &workspace.brush_hardness_slider,
                        format!("{:.0}%", brush_hardness * 100.0),
                        cx,
                    ))
                    .child(property_slider(
                        "Spacing",
                        &workspace.brush_spacing_slider,
                        format!("{:.0}%", brush_spacing * 100.0),
                        cx,
                    ))
                    .child(property_slider(
                        "Stabilization",
                        &workspace.brush_stabilization_slider,
                        format!("{:.0}%", brush_stabilization * 100.0),
                        cx,
                    )),
            )
    }
}

use std::sync::Arc;

#[derive(Clone)]
struct LayerData {
    visible: bool,
    opacity: f32,
    name: String,
    render_cache: std::collections::HashMap<(u32, u32), Arc<RenderImage>>,
    tile_keys: Vec<(u32, u32)>,
}

struct ThumbnailElement {
    render_cache: std::collections::HashMap<(u32, u32), Arc<RenderImage>>,
    tile_keys: Vec<(u32, u32)>,
    doc_size: Size<u32>,
}

impl IntoElement for ThumbnailElement {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for ThumbnailElement {
    type RequestLayoutState = ();
    type PrepaintState = ();

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
                    width: px(22.).into(),
                    height: px(22.).into(),
                },
                ..Default::default()
            },
            vec![],
            cx,
        );
        (layout_id, ())
    }

    #[allow(clippy::unused_unit)]
    fn prepaint(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _layout: &mut Self::RequestLayoutState,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
        ()
    }

    fn paint(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        _cx: &mut App,
    ) {
        if !self.tile_keys.is_empty() {
            // Find content bounding box (coordinates of allocated tiles)
            let mut min_tx = u32::MAX;
            let mut max_tx = 0;
            let mut min_ty = u32::MAX;
            let mut max_ty = 0;
            for &(tx, ty) in &self.tile_keys {
                if tx < min_tx {
                    min_tx = tx;
                }
                if tx > max_tx {
                    max_tx = tx;
                }
                if ty < min_ty {
                    min_ty = ty;
                }
                if ty > max_ty {
                    max_ty = ty;
                }
            }

            let min_x = min_tx * crate::tile::TILE_SIZE;
            let max_x = (max_tx + 1) * crate::tile::TILE_SIZE;
            let min_y = min_ty * crate::tile::TILE_SIZE;
            let max_y = (max_ty + 1) * crate::tile::TILE_SIZE;

            let content_w = (max_x - min_x) as f32;
            let content_h = (max_y - min_y) as f32;

            let bounds_w: f32 = bounds.size.width.into();
            let bounds_h: f32 = bounds.size.height.into();

            let scale_x = bounds_w / content_w;
            let scale_y = bounds_h / content_h;
            let scale = scale_x.min(scale_y);

            let offset_x = (min_x as f32) * scale;
            let offset_y = (min_y as f32) * scale;

            let target_width = px(self.doc_size.width as f32 * scale);
            let target_height = px(self.doc_size.height as f32 * scale);

            let draw_bounds = Bounds {
                origin: Point {
                    x: bounds.origin.x - px(offset_x)
                        + (bounds.size.width - px(content_w * scale)) / 2.,
                    y: bounds.origin.y - px(offset_y)
                        + (bounds.size.height - px(content_h * scale)) / 2.,
                },
                size: Size {
                    width: target_width,
                    height: target_height,
                },
            };

            // Clip the rendering bounds to the visual boundary of the thumbnail box
            window.with_content_mask(Some(ContentMask { bounds }), |window| {
                for &(tx, ty) in &self.tile_keys {
                    if let Some(render_image) = self.render_cache.get(&(tx, ty)) {
                        let tile_x = tx * crate::tile::TILE_SIZE;
                        let tile_y = ty * crate::tile::TILE_SIZE;
                        let tile_w = (crate::tile::TILE_SIZE as f32) * scale;
                        let tile_h = (crate::tile::TILE_SIZE as f32) * scale;
                        let tile_draw_bounds = Bounds {
                            origin: Point {
                                x: draw_bounds.origin.x + px(tile_x as f32 * scale),
                                y: draw_bounds.origin.y + px(tile_y as f32 * scale),
                            },
                            size: Size {
                                width: px(tile_w),
                                height: px(tile_h),
                            },
                        };
                        let _ = window.paint_image(
                            tile_draw_bounds,
                            Corners::default(),
                            render_image.clone(),
                            0,
                            false,
                        );
                    }
                }
            });
        }
    }
}

#[derive(IntoElement)]
pub struct LayerPanel {
    pub workspace: WeakEntity<Workspace>,
}

impl RenderOnce for LayerPanel {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let workspace_entity = self.workspace.clone();
        let Some(workspace) = self.workspace.upgrade() else {
            return div();
        };
        let (
            layers_data,
            active_layer_index,
            layer_opacity_slider,
            dragging_layer_index,
            _animated_swap_offset,
            drop_target_index,
            doc_size,
        ) = {
            let doc_entity = workspace.read(cx).document.clone();
            let layer_entities: Vec<Entity<Layer>> = doc_entity.read(cx).layers.clone();
            let active_layer_idx = doc_entity.read(cx).active_layer_index;
            let doc_sz = doc_entity.read(cx).size;

            let layers_list: Vec<LayerData> = layer_entities
                .iter()
                .map(|l| {
                    let mut render_cache = std::collections::HashMap::new();
                    let mut tile_keys = Vec::new();
                    l.update(cx, |layer, _cx| {
                        let Layer::Raster(raster) = layer;
                        for &coords in raster.tiles.tiles.keys() {
                            let entry = raster.render_cache.entry(coords).or_insert_with(|| {
                                let tile = raster.tiles.tiles.get(&coords).unwrap();
                                tile.build_render_image()
                            });
                            render_cache.insert(coords, entry.clone());
                            tile_keys.push(coords);
                        }
                    });
                    let layer_read = l.read(cx);
                    LayerData {
                        visible: layer_read.visible(),
                        opacity: layer_read.opacity(),
                        name: match layer_read {
                            Layer::Raster(r) => r.name.clone(),
                        },
                        render_cache,
                        tile_keys,
                    }
                })
                .collect();

            let workspace_ref = workspace.read(cx);
            (
                layers_list,
                active_layer_idx,
                workspace_ref.layer_opacity_slider.clone(),
                workspace_ref.dragging_layer_index,
                workspace_ref.animated_swap_offset,
                workspace_ref.drop_target_index,
                doc_sz,
            )
        };

        let active_layer_opacity = if let Some(active_layer) = layers_data.get(active_layer_index) {
            active_layer.opacity
        } else {
            1.0
        };

        // Update slider state cleanly
        layer_opacity_slider.update(cx, |state, _cx| {
            if (state.value().end() - active_layer_opacity).abs() > 0.001 {
                *state = gpui_component::slider::SliderState::new()
                    .min(0.0)
                    .max(1.0)
                    .step(0.01)
                    .default_value(active_layer_opacity);
            }
        });

        let layer_count = layers_data.len();
        let cx_ref = &*cx;

        div()
            .flex()
            .flex_col()
            .child(
                div()
                    .p(px(8.))
                    .border_t(px(1.))
                    .border_b(px(1.))
                    .border_color(cx.theme().border)
                    .flex()
                    .justify_between()
                    .items_center()
                    .child("Layers")
                    .child(
                        div()
                            .flex()
                            .gap(px(4.))
                            .child({
                                let workspace_entity = workspace_entity.clone();
                                icon_button("layer-add", "+", move |_, _, cx| {
                                    workspace_entity
                                        .update(cx, |this, cx| {
                                            this.document.update(cx, |doc, cx| {
                                                let name =
                                                    format!("Layer {}", doc.layers.len() + 1);
                                                doc.add_layer(&name, cx);
                                                cx.notify();
                                            });
                                        })
                                        .ok();
                                })
                            })
                            .child({
                                let workspace_entity = workspace_entity.clone();
                                icon_button("layer-del", "-", move |_, _, cx| {
                                    workspace_entity
                                        .update(cx, |this, cx| {
                                            this.document.update(cx, |doc, cx| {
                                                doc.delete_layer(doc.active_layer_index, cx);
                                                cx.notify();
                                            });
                                        })
                                        .ok();
                                })
                            })
                            .child({
                                let workspace_entity = workspace_entity.clone();
                                icon_button("layer-up", "↑", move |_, _, cx| {
                                    workspace_entity
                                        .update(cx, |this, cx| {
                                            this.document.update(cx, |doc, cx| {
                                                let idx = doc.active_layer_index;
                                                if idx > 0 {
                                                    doc.move_layer(idx, idx - 1);
                                                }
                                                cx.notify();
                                            });
                                        })
                                        .ok();
                                })
                            })
                            .child({
                                let workspace_entity = workspace_entity.clone();
                                icon_button("layer-down", "↓", move |_, _, cx| {
                                    workspace_entity
                                        .update(cx, |this, cx| {
                                            this.document.update(cx, |doc, cx| {
                                                let idx = doc.active_layer_index;
                                                if idx < doc.layers.len() - 1 {
                                                    doc.move_layer(idx, idx + 1);
                                                }
                                                cx.notify();
                                            });
                                        })
                                        .ok();
                                })
                            }),
                    ),
            )
            .child(
                div()
                    .p(px(8.))
                    .border_b(px(1.))
                    .border_color(cx.theme().border)
                    .child(property_slider(
                        "Layer Opacity",
                        &layer_opacity_slider,
                        format!("{:.0}%", active_layer_opacity * 100.0),
                        cx,
                    )),
            )
            .child(
                div()
                    .flex_grow()
                    .children((0..layer_count).rev().map(move |idx| {
                        let layer_data = layers_data[idx].clone();
                        let visible = layer_data.visible;
                        let name = layer_data.name;
                        let render_cache = layer_data.render_cache;
                        let tile_keys = layer_data.tile_keys;
                        let is_active = idx == active_layer_index;
                        let workspace_entity = workspace_entity.clone();
                        let is_dragging_this = dragging_layer_index == Some(idx);
                        div()
                            .id(("layer", idx))
                            .p(px(6.))
                            .flex()
                            .items_center()
                            .justify_between()
                            .bg(if is_dragging_this || is_active {
                                cx_ref.theme().accent
                            } else {
                                cx_ref.theme().background
                            })
                            .hover(|s| s.bg(cx_ref.theme().border))
                            .on_mouse_down(MouseButton::Left, {
                                let workspace_entity = workspace_entity.clone();
                                move |_, _, cx| {
                                    workspace_entity
                                        .update(cx, |this, cx| {
                                            this.dragging_layer_index = Some(idx);
                                            this.drop_target_index = Some(idx);
                                            this.document.update(cx, |doc, cx| {
                                                doc.active_layer_index = idx;
                                                cx.notify();
                                            });
                                            cx.notify();
                                        })
                                        .ok();
                                }
                            })
                            .on_mouse_move({
                                let workspace_entity = workspace_entity.clone();
                                move |_, _, cx| {
                                    workspace_entity
                                        .update(cx, |this, cx| {
                                            if this.dragging_layer_index.is_some()
                                                && this.drop_target_index != Some(idx)
                                            {
                                                this.drop_target_index = Some(idx);
                                                cx.notify();
                                            }
                                        })
                                        .ok();
                                }
                            })
                            .on_mouse_up(MouseButton::Left, {
                                let workspace_entity = workspace_entity.clone();
                                move |_, _, cx| {
                                    workspace_entity
                                        .update(cx, |this, cx| {
                                            if let Some((dragged_idx, target_idx)) = this
                                                .dragging_layer_index
                                                .zip(this.drop_target_index)
                                                .filter(|(dragged, target)| dragged != target)
                                            {
                                                this.document.update(cx, |doc, cx| {
                                                    doc.move_layer(dragged_idx, target_idx);
                                                    cx.notify();
                                                });
                                            }
                                            this.dragging_layer_index = None;
                                            this.drop_target_index = None;
                                            cx.notify();
                                        })
                                        .ok();
                                }
                            })
                            .relative()
                            .child({
                                let is_target = dragging_layer_index.is_some()
                                    && drop_target_index == Some(idx)
                                    && dragging_layer_index != Some(idx);
                                let is_top = is_target
                                    && dragging_layer_index
                                        .map(|dragged| idx > dragged)
                                        .unwrap_or(false);
                                div()
                                    .absolute()
                                    .left(px(8.))
                                    .right(px(8.))
                                    .h(px(2.))
                                    .bg(gpui::white())
                                    .when(is_top, |s| s.top(px(-1.)))
                                    .when(!is_top, |s| s.bottom(px(-1.)))
                                    .when(!is_target, |s| s.bg(gpui::transparent_black()))
                            })
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.))
                                    // Layer Thumbnail Preview block
                                    .child(
                                        div()
                                            .size(px(24.))
                                            .bg(rgb(0xcccccc)) // Gray checkered base
                                            .border(px(1.))
                                            .border_color(cx_ref.theme().border)
                                            .rounded(px(2.))
                                            .relative()
                                            .child(
                                                // checker pattern blocks
                                                div()
                                                    .absolute()
                                                    .top_0()
                                                    .left_0()
                                                    .size(px(12.))
                                                    .bg(rgb(0xaaaaaa)),
                                            )
                                            .child(
                                                div()
                                                    .absolute()
                                                    .bottom_0()
                                                    .right_0()
                                                    .size(px(12.))
                                                    .bg(rgb(0xaaaaaa)),
                                            )
                                            .child(div().absolute().top_0().left_0().child(
                                                ThumbnailElement {
                                                    render_cache,
                                                    tile_keys,
                                                    doc_size,
                                                },
                                            )),
                                    )
                                    .child(div().text_size(px(12.)).child(name)),
                            )
                            .child({
                                // Custom vector line icon for eye visibility toggle
                                let workspace_entity = workspace_entity.clone();
                                div()
                                    .id(("visible-btn", idx))
                                    .size(px(20.))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .hover(|s| s.bg(cx_ref.theme().border))
                                    .rounded(px(4.))
                                    .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                        // Stop propagation by not triggering row selection
                                        cx.stop_propagation();
                                    })
                                    .on_click(move |_, _, cx| {
                                        workspace_entity
                                            .update(cx, |this, cx| {
                                                this.document.update(cx, |doc, cx| {
                                                    doc.toggle_visibility(idx, cx);
                                                });
                                            })
                                            .ok();
                                    })
                                    .child(
                                        svg()
                                            .path(if visible {
                                                "icons/eye.svg"
                                            } else {
                                                "icons/eye-slash.svg"
                                            })
                                            .size(px(14.))
                                            .text_color(if visible {
                                                cx_ref.theme().foreground
                                            } else {
                                                cx_ref.theme().muted_foreground
                                            }),
                                    )
                            })
                    })),
            )
    }
}
