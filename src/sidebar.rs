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
        let (layers_data, active_layer_index, layer_opacity_slider, dragging_layer_index, _animated_swap_offset, drop_target_index) = {
            let workspace_ref = workspace.read(cx);
            let doc = workspace_ref.document.read(cx);
            let layers_list: Vec<(bool, f32, String, Entity<Layer>)> = doc.layers.iter().map(|l| {
                let layer_read = l.read(cx);
                (layer_read.visible(), layer_read.opacity(), match layer_read { Layer::Raster(r) => r.name.clone() }, l.clone())
            }).collect();
            (layers_list, doc.active_layer_index, workspace_ref.layer_opacity_slider.clone(), workspace_ref.dragging_layer_index, workspace_ref.animated_swap_offset, workspace_ref.drop_target_index)
        };

        let active_layer_opacity = if let Some(active_layer) = layers_data.get(active_layer_index) {
            active_layer.1
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
                    ))
            )
            .child(
                div()
                    .flex_grow()
                    .children((0..layer_count).rev().map(move |idx| {
                        let (visible, opacity, name, _layer_entity) = layers_data[idx].clone();
                        let is_active = idx == active_layer_index;
                        let workspace_entity = workspace_entity.clone();
                        let is_dragging_this = dragging_layer_index == Some(idx);
                        div()
                            .id(("layer", idx))
                            .p(px(8.))
                            .flex()
                            .items_center()
                            .justify_between()
                            .bg(if is_dragging_this {
                                cx_ref.theme().accent
                            } else if is_active {
                                // Highly distinct select background style (theme accent/primary)
                                cx_ref.theme().accent
                            } else {
                                // Different from the sidebar panel background (muted vs background)
                                cx_ref.theme().background
                            })
                            .hover(|s| s.bg(cx_ref.theme().border)) // Subtle hover background (border color is muted, perfect for subtle hover)
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
                                            if this.dragging_layer_index.is_some() {
                                                if this.drop_target_index != Some(idx) {
                                                    this.drop_target_index = Some(idx);
                                                    cx.notify();
                                                }
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
                                            if let (Some(dragged_idx), Some(target_idx)) = (this.dragging_layer_index, this.drop_target_index) {
                                                if dragged_idx != target_idx {
                                                    this.document.update(cx, |doc, cx| {
                                                        doc.move_layer(dragged_idx, target_idx);
                                                        cx.notify();
                                                    });
                                                }
                                            }
                                            this.dragging_layer_index = None;
                                            this.drop_target_index = None;
                                            cx.notify();
                                        })
                                        .ok();
                                }
                            })
                            .relative() // Enable absolute placement inside this element
                            .child({
                                let is_target = dragging_layer_index.is_some() && drop_target_index == Some(idx) && dragging_layer_index != Some(idx);
                                let is_top = is_target && dragging_layer_index.map(|dragged| idx < dragged).unwrap_or(false);
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
                                    .child({
                                        let workspace_entity = workspace_entity.clone();
                                        icon_button(
                                            if visible { "layer-visible" } else { "layer-hidden" },
                                            if visible { "👁" } else { "👁‍🗨" },
                                            move |_, _, cx| {
                                                workspace_entity
                                                    .update(cx, |this, cx| {
                                                        this.document.update(cx, |doc, cx| {
                                                            doc.toggle_visibility(idx, cx);
                                                        });
                                                    })
                                                    .ok();
                                            },
                                        )
                                    })
                                    .child(name)
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(4.))
                                    .child(
                                        div()
                                            .text_size(px(10.))
                                            .text_color(cx_ref.theme().muted_foreground)
                                            .child(format!("{:.0}%", opacity * 100.0))
                                    )
                            )
                    })),
            )
    }
}
