use crate::document::Layer;
use crate::tool::Tool;
use crate::ui_components::{icon_button, property_slider, tool_button};
use crate::workspace::Workspace;
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
        let workspace = workspace.read(cx);
        let doc = workspace.document.read(cx);
        let layers = &doc.layers;
        let active_layer_index = doc.active_layer_index;
        let layer_count = layers.len();
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
                    .flex_grow()
                    .children(
                        (0..layer_count).rev().map(move |idx| {
                            let layer_entity = layers[idx].clone();
                            let is_active = idx == active_layer_index;
                            let workspace_entity = workspace_entity.clone();
                            let name = match layer_entity.read(cx_ref) {
                                Layer::Raster(r) => r.name.clone(),
                            };
                            div()
                                .id(("layer", idx))
                                .p(px(8.))
                                .bg(if is_active {
                                    cx_ref.theme().border
                                } else {
                                    cx_ref.theme().muted
                                })
                                .hover(|s| s.bg(cx_ref.theme().accent))
                                .on_click(move |_, _, cx| {
                                    workspace_entity
                                        .update(cx, |this, cx| {
                                            this.document.update(cx, |doc, cx| {
                                                doc.active_layer_index = idx;
                                                cx.notify();
                                            });
                                        })
                                        .ok();
                                })
                                .child(name)
                        }),
                    ),
            )
    }
}
