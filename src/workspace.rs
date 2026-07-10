use crate::sidebar::{LayerPanel, PropertiesPanel, Toolbox};
use crate::tool::{Tool, ToolEvent, ToolState};
use crate::ui_components::*;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::ActiveTheme;

actions!(sienna, [Undo, Redo, NewProject]);

use crate::project_modal::*;

pub struct Workspace {
    pub(crate) document: Entity<crate::document::Document>,
    pub(crate) tool_state: Entity<ToolState>,
    modal: Option<Entity<NewProjectModal>>,
    focus_handle: FocusHandle,
    pub(crate) color_picker_state: Entity<gpui_component::color_picker::ColorPickerState>,
    pub(crate) brush_size_slider: Entity<gpui_component::slider::SliderState>,
    pub(crate) brush_opacity_slider: Entity<gpui_component::slider::SliderState>,
    pub(crate) brush_flow_slider: Entity<gpui_component::slider::SliderState>,
    pub(crate) brush_hardness_slider: Entity<gpui_component::slider::SliderState>,
    pub(crate) brush_spacing_slider: Entity<gpui_component::slider::SliderState>,
    pub(crate) brush_stabilization_slider: Entity<gpui_component::slider::SliderState>,
    pub(crate) dragging_layer_index: Option<usize>,
    pub(crate) animated_swap_offset: f32,
    pub(crate) layer_opacity_slider: Entity<gpui_component::slider::SliderState>,
    pub(crate) drop_target_index: Option<usize>,
}

impl Workspace {
    pub fn new(
        document: Entity<crate::document::Document>,
        tool_state: Entity<ToolState>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let window_handle = window.window_handle();
        let initial_color = tool_state.read(cx).active_color;
        let color_picker_state = cx.new(|cx| {
            gpui_component::color_picker::ColorPickerState::new(window, cx)
                .default_value(Hsla::from(initial_color))
        });

        cx.subscribe(&color_picker_state, |this, _entity, event, cx| {
            let gpui_component::color_picker::ColorPickerEvent::Change(color) = event;
            if let Some(color) = color {
                let color_rgba = Rgba::from(*color);
                this.tool_state.update(cx, |ts, cx| {
                    if ts.active_color != color_rgba {
                        ts.active_color = color_rgba;
                        cx.emit(ToolEvent::ColorChanged(color_rgba));
                    }
                });
            }
        })
        .detach();

        let color_picker_state_clone = color_picker_state.clone();
        cx.subscribe(&tool_state, move |_this, _entity, event, cx| {
            let ToolEvent::ColorChanged(color) = event;
            let color = *color;
            let color_picker_state = color_picker_state_clone.clone();
            let _ = window_handle.update(cx, |_, window, cx| {
                color_picker_state.update(cx, |state, cx| {
                    let hsla = Hsla::from(color);
                    if state.value() != Some(hsla) {
                        state.set_value(hsla, window, cx);
                    }
                });
            });
        })
        .detach();

        // Only notify when document actually changes via explicit updates, not on every read
        cx.observe(&tool_state, |_, _, cx| {
            cx.notify();
        })
        .detach();

        let ts = tool_state.read(cx);
        let brush_size = ts.brush_size;
        let brush_opacity = ts.brush_opacity;
        let brush_flow = ts.brush_flow;
        let brush_hardness = ts.brush_hardness;
        let brush_spacing = ts.brush_spacing;
        let brush_stabilization = ts.brush_stabilization;

        let brush_size_slider = cx.new(|_| {
            gpui_component::slider::SliderState::new()
                .min(1.0)
                .max(500.0)
                .step(1.0)
                .default_value(brush_size)
        });
        let brush_opacity_slider = cx.new(|_| {
            gpui_component::slider::SliderState::new()
                .min(0.0)
                .max(1.0)
                .step(0.01)
                .default_value(brush_opacity)
        });
        let brush_flow_slider = cx.new(|_| {
            gpui_component::slider::SliderState::new()
                .min(0.0)
                .max(1.0)
                .step(0.01)
                .default_value(brush_flow)
        });
        let brush_hardness_slider = cx.new(|_| {
            gpui_component::slider::SliderState::new()
                .min(0.0)
                .max(1.0)
                .step(0.01)
                .default_value(brush_hardness)
        });
        let brush_spacing_slider = cx.new(|_| {
            gpui_component::slider::SliderState::new()
                .min(0.01)
                .max(2.0)
                .step(0.01)
                .default_value(brush_spacing)
        });
        let brush_stabilization_slider = cx.new(|_| {
            gpui_component::slider::SliderState::new()
                .min(0.0)
                .max(0.95)
                .step(0.01)
                .default_value(brush_stabilization)
        });

        let layer_opacity_slider = cx.new(|_| {
            gpui_component::slider::SliderState::new()
                .min(0.0)
                .max(1.0)
                .step(0.01)
                .default_value(1.0)
        });

        let ws = Self {
            document,
            tool_state: tool_state.clone(),
            modal: None,
            focus_handle: cx.focus_handle(),
            color_picker_state,
            brush_size_slider: brush_size_slider.clone(),
            brush_opacity_slider: brush_opacity_slider.clone(),
            brush_flow_slider: brush_flow_slider.clone(),
            brush_hardness_slider: brush_hardness_slider.clone(),
            brush_spacing_slider: brush_spacing_slider.clone(),
            brush_stabilization_slider: brush_stabilization_slider.clone(),
            dragging_layer_index: None,
            animated_swap_offset: 0.0,
            layer_opacity_slider: layer_opacity_slider.clone(),
            drop_target_index: None,
        };

        // Subscribe to sliders

        cx.subscribe(&brush_size_slider, move |this, _entity, event, cx| {
            let gpui_component::slider::SliderEvent::Change(val) = event;
            let val: f32 = val.end();
            this.tool_state.update(cx, |ts, _cx| {
                ts.brush_size = val;
            });
        })
        .detach();

        cx.subscribe(&brush_opacity_slider, move |this, _entity, event, cx| {
            let gpui_component::slider::SliderEvent::Change(val) = event;
            let val: f32 = val.end();
            this.tool_state.update(cx, |ts, _cx| {
                ts.brush_opacity = val;
            });
        })
        .detach();

        cx.subscribe(&brush_flow_slider, move |this, _entity, event, cx| {
            let gpui_component::slider::SliderEvent::Change(val) = event;
            let val: f32 = val.end();
            this.tool_state.update(cx, |ts, _cx| {
                ts.brush_flow = val;
            });
        })
        .detach();

        cx.subscribe(&brush_hardness_slider, move |this, _entity, event, cx| {
            let gpui_component::slider::SliderEvent::Change(val) = event;
            let val: f32 = val.end();
            this.tool_state.update(cx, |ts, _cx| {
                ts.brush_hardness = val;
            });
        })
        .detach();

        cx.subscribe(&brush_spacing_slider, move |this, _entity, event, cx| {
            let gpui_component::slider::SliderEvent::Change(val) = event;
            let val: f32 = val.end();
            this.tool_state.update(cx, |ts, _cx| {
                ts.brush_spacing = val;
            });
        })
        .detach();

        cx.subscribe(
            &brush_stabilization_slider,
            move |this, _entity, event, cx| {
                let gpui_component::slider::SliderEvent::Change(val) = event;
                let val: f32 = val.end();
                this.tool_state.update(cx, |ts, _cx| {
                    ts.brush_stabilization = val;
                });
            },
        )
        .detach();

        cx.subscribe(&layer_opacity_slider, move |this, _entity, event, cx| {
            let gpui_component::slider::SliderEvent::Change(val) = event;
            let val: f32 = val.end();
            this.document.update(cx, |doc, cx| {
                let active_idx = doc.active_layer_index;
                doc.set_opacity(active_idx, val, cx);
            });
        })
        .detach();

        ws
    }

    fn new_project(&mut self, _: &NewProject, window: &mut Window, cx: &mut Context<Self>) {
        let workspace = cx.entity().downgrade();
        let modal = cx.new(|cx| {
            let ws_create = workspace.clone();
            let ws_cancel = workspace.clone();
            NewProjectModal::new(
                window,
                cx,
                move |w, h, _window, cx| {
                    let _ = ws_create.update(cx, |this, cx| {
                        this.document.update(cx, |doc, cx| {
                            *doc = crate::document::Document::new(
                                Size {
                                    width: w,
                                    height: h,
                                },
                                cx,
                            );
                            cx.notify();
                        });
                        this.modal = None;
                        cx.notify();
                    });
                },
                move |_window, cx| {
                    let _ = ws_cancel.update(cx, |this, cx| {
                        this.modal = None;
                        cx.notify();
                    });
                },
            )
        });
        self.modal = Some(modal);
        cx.notify();
    }

    fn save(&mut self, _event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let mut doc_data = self.document.read(cx).to_data(cx);
        for layer in &mut doc_data.layers {
            let crate::document::LayerData::Raster { tiles, .. } = layer;
            tiles.swap_rb_channels();
        }
        cx.spawn(|_this, cx: &mut AsyncApp| {
            let cx = cx.clone();
            async move {
                let file = rfd::AsyncFileDialog::new()
                    .add_filter("Sienna", &["sienna"])
                    .save_file()
                    .await;
                if let Some(file) = file {
                    let path = file.path().to_path_buf();
                    cx.background_spawn(async move {
                        let encoded = bincode::serialize(&doc_data).unwrap();
                        std::fs::write(path, encoded).unwrap();
                    })
                    .await;
                }
            }
        })
        .detach();
    }

    fn open(&mut self, _event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let document_entity = self.document.downgrade();
        cx.spawn(|_this, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                let file = rfd::AsyncFileDialog::new()
                    .add_filter("Sienna", &["sienna"])
                    .pick_file()
                    .await;
                if let Some(file) = file {
                    let path = file.path().to_path_buf();
                    let mut data = cx
                        .background_spawn(async move {
                            let bytes = std::fs::read(path).unwrap();
                            let data: crate::document::DocumentData =
                                bincode::deserialize(&bytes).unwrap();
                            data
                        })
                        .await;

                    for layer in &mut data.layers {
                        let crate::document::LayerData::Raster { tiles, .. } = layer;
                        tiles.swap_rb_channels();
                    }

                    document_entity
                        .update(&mut cx, |doc, cx| {
                            *doc = crate::document::Document::from_data(data, cx);
                            cx.notify();
                        })
                        .ok();

                    // Render cache will be built on demand in paint
                }
            }
        })
        .detach();
    }

    fn import_image(&mut self, _event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let document_entity = self.document.downgrade();
        let doc_size = self.document.read(cx).size;

        cx.spawn(move |_this, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                let file = rfd::AsyncFileDialog::new()
                    .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                    .pick_file()
                    .await;
                if let Some(file) = file {
                    let path = file.path().to_path_buf();
                    let layer_name = file.file_name();

                    let mut layer_data = cx
                        .background_spawn(async move {
                            let img = image::open(path).expect("Failed to open image");
                            // Resize to document size
                            let resized = img
                                .resize_exact(
                                    doc_size.width,
                                    doc_size.height,
                                    image::imageops::FilterType::Lanczos3,
                                )
                                .to_rgba8();
                            let raw_pixels = resized.into_raw();
                            let tiles = crate::tile::TileGrid::from_monolithic(
                                doc_size.width,
                                doc_size.height,
                                &raw_pixels,
                            );
                            crate::document::LayerData::Raster {
                                name: layer_name,
                                visible: true,
                                opacity: 1.0,
                                tiles,
                            }
                        })
                        .await;

                    let crate::document::LayerData::Raster { tiles, .. } = &mut layer_data;
                    tiles.swap_rb_channels();

                    let _ = document_entity
                        .update(&mut cx, |doc, cx| {
                            let layer = cx.new(|_cx| match layer_data {
                                crate::document::LayerData::Raster {
                                    name,
                                    visible,
                                    opacity,
                                    tiles,
                                } => crate::document::Layer::Raster(crate::document::RasterLayer {
                                    name,
                                    visible,
                                    opacity,
                                    tiles,
                                    render_cache: std::collections::HashMap::new(),
                                }),
                            });
                            doc.layers.insert(0, layer.clone());
                            doc.active_layer_index = 0;
                            cx.notify();
                            layer
                        })
                        .ok();

                    // Render cache will be built on demand in paint
                }
            }
        })
        .detach();
    }

    fn undo(&mut self, _: &Undo, _window: &mut Window, cx: &mut Context<Self>) {
        self.document.update(cx, |doc, cx| {
            doc.undo(cx);
            cx.notify();
        });
    }

    fn redo(&mut self, _: &Redo, _window: &mut Window, cx: &mut Context<Self>) {
        self.document.update(cx, |doc, cx| {
            doc.redo(cx);
            cx.notify();
        });
    }

    fn pick_color(
        document: &Entity<crate::document::Document>,
        tool_state: &Entity<ToolState>,
        screen_position: Point<Pixels>,
        cx: &mut App,
    ) {
        let doc_size = document.read(cx).size;
        let transform = document.read(cx).transform;
        let origin = Point {
            x: px(48.0),
            y: px(40.0),
        };
        let screen_pos = screen_position - origin;

        let canvas_pos = Point {
            x: (screen_pos.x.to_f64() as f32 - transform.offset.x) / transform.scale,
            y: (screen_pos.y.to_f64() as f32 - transform.offset.y) / transform.scale,
        };

        let x = canvas_pos.x as i32;
        let y = canvas_pos.y as i32;

        if x >= 0 && x < doc_size.width as i32 && y >= 0 && y < doc_size.height as i32 {
            let mut picked_color = None;
            let layers = document.read(cx).layers.clone();
            for layer_entity in layers.iter().rev() {
                let layer = layer_entity.read(cx);
                let crate::document::Layer::Raster(raster) = layer;
                if raster.visible {
                    let color = raster.tiles.get_pixel(x as u32, y as u32);
                    let a = color[3];
                    if a > 0 {
                        picked_color = Some(Rgba {
                            r: color[2] as f32 / 255.0,
                            g: color[1] as f32 / 255.0,
                            b: color[0] as f32 / 255.0,
                            a: a as f32 / 255.0,
                        });
                        break;
                    }
                }
            }

            if let Some(color) = picked_color {
                tool_state.update(cx, |ts, cx| {
                    if ts.active_color != color {
                        ts.active_color = color;
                        cx.emit(ToolEvent::ColorChanged(color));
                        cx.notify();
                    }
                });
            }
        }
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let active_tool = self.tool_state.read(cx).active_tool;

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::undo))
            .on_action(cx.listener(Self::redo))
            .on_action(cx.listener(Self::new_project))
            .child(
                div()
                    .h(px(40.))
                    .w_full()
                    .bg(cx.theme().muted)
                    .border_b(px(1.))
                    .border_color(cx.theme().border)
                    .px(px(8.))
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .child(
                        div()
                            .text_size(px(14.))
                            .font_weight(FontWeight::BOLD)
                            .child("SIENNA"),
                    )
                    .child(menu_button(
                        "new-btn",
                        "New",
                        cx.listener(|this, _, window, cx| {
                            this.new_project(&NewProject, window, cx)
                        }),
                    ))
                    .child(menu_button("open-btn", "Open", cx.listener(Self::open)))
                    .child(menu_button("save-btn", "Save", cx.listener(Self::save)))
                    .child(menu_button(
                        "import-btn",
                        "Import",
                        cx.listener(Self::import_image),
                    ))
                    .child(div().w(px(12.)))
                    .child(menu_button(
                        "undo-btn",
                        "Undo",
                        cx.listener(|this, _, window, cx| this.undo(&Undo, window, cx)),
                    ))
                    .child(menu_button(
                        "redo-btn",
                        "Redo",
                        cx.listener(|this, _, window, cx| this.redo(&Redo, window, cx)),
                    )),
            )
            .child(
                div()
                    .flex_grow()
                    .flex()
                    .child(Toolbox {
                        workspace: cx.entity().downgrade(),
                    })
                    .child(
                        div()
                            .flex_grow()
                            .h_full()
                            .on_scroll_wheel(cx.listener(
                                |this, event: &ScrollWheelEvent, _window, cx| {
                                    this.document.update(cx, |doc, cx| {
                                        let delta = match event.delta {
                                            ScrollDelta::Pixels(p) => p,
                                            ScrollDelta::Lines(l) => l.map(|v| px(v * 20.0)),
                                        };

                                        if event.modifiers.secondary() {
                                            let factor =
                                                if delta.y.to_f64() > 0.0 { 1.1 } else { 0.9 };
                                            doc.transform.scale *= factor;
                                            doc.transform.scale =
                                                doc.transform.scale.clamp(0.01, 100.0);
                                        } else {
                                            doc.transform.offset.x += delta.x.to_f64() as f32;
                                            doc.transform.offset.y += delta.y.to_f64() as f32;
                                        }
                                        cx.notify();
                                    });
                                },
                            ))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, event: &MouseDownEvent, _window, cx| {
                                    let (document, tool_state) =
                                        (this.document.clone(), this.tool_state.clone());
                                    let tool = tool_state.read(cx).active_tool;
                                    if tool == Tool::Brush || tool == Tool::Eraser {
                                        let doc_size = document.read(cx).size;
                                        let transform = document.read(cx).transform;
                                        let layer_tiles = document
                                            .read(cx)
                                            .active_layer()
                                            .map(|l| match l.read(cx) {
                                                crate::document::Layer::Raster(r) => r.tiles.clone(),
                                            });
                                        if let Some(tiles) = layer_tiles {
                                            let origin = Point {
                                                x: px(48.0),
                                                y: px(40.0),
                                            };
                                            let screen_pos = event.position - origin;
                                            let canvas_pos = Point {
                                                x: (screen_pos.x.to_f64() as f32
                                                    - transform.offset.x)
                                                    / transform.scale,
                                                y: (screen_pos.y.to_f64() as f32
                                                    - transform.offset.y)
                                                    / transform.scale,
                                            };

                                             let (tx_points, rx_points): (tokio::sync::mpsc::UnboundedSender<Point<f32>>, tokio::sync::mpsc::UnboundedReceiver<Point<f32>>) = tokio::sync::mpsc::unbounded_channel();
                                             let (tx_updates, mut rx_updates): (tokio::sync::mpsc::UnboundedSender<crate::tool::StrokeUpdate>, tokio::sync::mpsc::UnboundedReceiver<crate::tool::StrokeUpdate>) = tokio::sync::mpsc::unbounded_channel();

                                            tool_state.update(cx, |ts, _cx| {
                                                let active_stroke = crate::tool::ActiveStroke {
                                                    tx_points: Some(tx_points),
                                                    width: doc_size.width,
                                                    height: doc_size.height,
                                                    composited_tiles: tiles.tiles.clone(),
                                                    render_cache: std::collections::HashMap::new(),
                                                    final_tiles: None,
                                                };
                                                ts.active_stroke = Some(active_stroke);
                                                ts.last_mouse_pos = Some(event.position);
                                            });

                                            let is_eraser = tool == Tool::Eraser;
                                            let brush_size = tool_state.read(cx).brush_size;
                                            let brush_opacity = tool_state.read(cx).brush_opacity;
                                            let brush_flow = tool_state.read(cx).brush_flow;
                                            let brush_hardness = tool_state.read(cx).brush_hardness;
                                            let brush_spacing = tool_state.read(cx).brush_spacing;
                                            let brush_stabilization = tool_state.read(cx).brush_stabilization;
                                            let active_color = tool_state.read(cx).active_color;

                                            cx.background_spawn(async move {
                                                let mut acc = crate::stroke::StrokeAccumulator::begin(
                                                    doc_size.width,
                                                    doc_size.height,
                                                    tiles,
                                                    brush_size,
                                                    brush_opacity,
                                                    brush_flow,
                                                    brush_hardness,
                                                    brush_spacing,
                                                    brush_stabilization,
                                                    active_color,
                                                    is_eraser,
                                                );

                                                let mut rx_points = rx_points;
                                                while let Some(pos) = rx_points.recv().await {
                                                    let dirty_coords = acc.feed(pos);
                                                    if !dirty_coords.is_empty() {
                                                        let mut changed_tiles = std::collections::HashMap::new();
                                                        for coords in dirty_coords {
                                                            if let Some(tile) = acc.stroke_buffer.composited.tiles.get(&coords) {
                                                                let render_image = tile.build_render_image();
                                                                changed_tiles.insert(coords, (tile.clone(), render_image));
                                                            }
                                                        }
                                                        tx_updates.send(crate::tool::StrokeUpdate::Tiles(changed_tiles)).ok();
                                                    }
                                                }
                                                let (before, after) = acc.finalize();
                                                tx_updates.send(crate::tool::StrokeUpdate::Finished(before, after)).ok();
                                            }).detach();

                                            let tool_state_handle = tool_state.downgrade();
                                            let document_handle = document.downgrade();
                                            cx.spawn(move |_this, cx: &mut AsyncApp| {
                                                let mut cx = cx.clone();
                                                async move {
                                                    let mut final_tiles = None;
                                                    while let Some(update) = rx_updates.recv().await {
                                                        let mut is_finished = false;
                                                        if let crate::tool::StrokeUpdate::Finished(before, after) = &update {
                                                            final_tiles = Some((before.clone(), after.clone()));
                                                            is_finished = true;
                                                        }

                                                        let _ = tool_state_handle.update(&mut cx, |ts, cx| {
                                                            if let Some(ref mut stroke) = ts.active_stroke {
                                                                match &update {
                                                                    crate::tool::StrokeUpdate::Tiles(tiles) => {
                                                                        for (coords, (tile, render_image)) in tiles {
                                                                            stroke.composited_tiles.insert(*coords, tile.clone());
                                                                            stroke.render_cache.insert(*coords, render_image.clone());
                                                                        }
                                                                        cx.notify();
                                                                    }
                                                                    crate::tool::StrokeUpdate::Finished(_, _) => {}
                                                                }
                                                            }
                                                        }).ok();

                                                        if is_finished {
                                                            break;
                                                        }
                                                    }

                                                    let mut stroke_render_cache = std::collections::HashMap::new();
                                                    let _ = tool_state_handle.update(&mut cx, |ts, _cx| {
                                                        if let Some(ref mut stroke) = ts.active_stroke {
                                                            stroke_render_cache = std::mem::take(&mut stroke.render_cache);
                                                        }
                                                    }).ok();

                                                    if let Some((before_tiles, after_tiles)) = final_tiles {
                                                        let active_layer_entity = document_handle.update(&mut cx, |doc, _cx| {
                                                            doc.active_layer().cloned()
                                                        }).ok().flatten();

                                                        if let Some(layer_entity) = active_layer_entity {
                                                            let has_changed = before_tiles != after_tiles;
                                                            let active_layer_index = document_handle.update(&mut cx, |doc, _cx| {
                                                                doc.active_layer_index
                                                            }).ok().unwrap_or(0);

                                                             layer_entity.update(&mut cx, |layer, cx| {
                                                                 let crate::document::Layer::Raster(raster) = layer;
                                                                 raster.tiles = after_tiles.clone();
                                                                 for (coords, render_image) in stroke_render_cache {
                                                                     raster.render_cache.insert(coords, render_image);
                                                                 }
                                                                 cx.notify();
                                                             });

                                                            if has_changed {
                                                                let diff_map = crate::tile::TileGrid::delta(&before_tiles, &after_tiles);
                                                                document_handle.update(&mut cx, |doc, cx| {
                                                                    doc.undo_stack.push(
                                                                        crate::document::Action::Paint {
                                                                            layer_index: active_layer_index,
                                                                            changed_tiles: diff_map,
                                                                        },
                                                                    );
                                                                    doc.redo_stack.clear();
                                                                    cx.notify();
                                                                }).ok();
                                                            }
                                                        }
                                                    }

                                                    tool_state_handle.update(&mut cx, |ts, cx| {
                                                        ts.active_stroke = None;
                                                        cx.notify();
                                                    }).ok();
                                                }
                                            }).detach();

                                            if let Some(tx) = tool_state.read(cx).active_stroke.as_ref().and_then(|s| s.tx_points.as_ref()) {
                                                tx.send(canvas_pos).ok();
                                            }
                                        }
                                    } else if tool == Tool::Move {
                                        tool_state.update(cx, |ts, _cx| {
                                            ts.last_mouse_pos = Some(event.position);
                                        });
                                    } else if tool == Tool::ColorPicker {
                                        Self::pick_color(
                                            &document,
                                            &tool_state,
                                            event.position,
                                            cx,
                                        );
                                    }
                                }),
                            )
                            .on_mouse_move(cx.listener(
                                |this, event: &MouseMoveEvent, _window, cx| {
                                    let (document, tool_state) =
                                        (this.document.clone(), this.tool_state.clone());
                                    let active_tool = tool_state.read(cx).active_tool;
                                    let transform = document.read(cx).transform;
                                    let origin = Point {
                                        x: px(48.0),
                                        y: px(40.0),
                                    };

                                    if event.pressed_button == Some(MouseButton::Left) {
                                        if active_tool == Tool::Move {
                                            if let Some(last_pos) =
                                                tool_state.read(cx).last_mouse_pos
                                            {
                                                let delta = event.position - last_pos;
                                                document.update(cx, |doc, cx| {
                                                    doc.transform.offset.x +=
                                                        delta.x.to_f64() as f32;
                                                    doc.transform.offset.y +=
                                                        delta.y.to_f64() as f32;
                                                    cx.notify();
                                                });
                                            }
                                            tool_state.update(cx, |ts, _cx| {
                                                ts.last_mouse_pos = Some(event.position);
                                            });
                                        } else if active_tool == Tool::ColorPicker {
                                            Self::pick_color(
                                                &document,
                                                &tool_state,
                                                event.position,
                                                cx,
                                            );
                                        } else if active_tool == Tool::Brush
                                            || active_tool == Tool::Eraser
                                        {
                                            let screen_pos = event.position - origin;
                                            let canvas_pos = Point {
                                                x: (screen_pos.x.to_f64() as f32
                                                    - transform.offset.x)
                                                    / transform.scale,
                                                y: (screen_pos.y.to_f64() as f32
                                                    - transform.offset.y)
                                                    / transform.scale,
                                            };

                                            tool_state.update(cx, |ts, cx| {
                                                if let Some(ref mut stroke) = ts.active_stroke {
                                                    stroke.tx_points.as_ref().map(|tx| tx.send(canvas_pos).ok());
                                                    ts.last_mouse_pos = Some(event.position);
                                                    cx.notify();
                                                }
                                            });
                                        }
                                    }
                                },
                            ))
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(|this, _event: &MouseUpEvent, _window, cx| {
                                    let tool = this.tool_state.read(cx).active_tool;
                                    if tool == Tool::Brush || tool == Tool::Eraser {
                                        this.tool_state.update(cx, |ts, cx| {
                                            ts.last_mouse_pos = None;
                                            if let Some(stroke) = ts.active_stroke.as_mut() {
                                                stroke.tx_points.take();
                                            }
                                            cx.notify();
                                        });
                                    } else if tool == Tool::Move {
                                        this.tool_state.update(cx, |ts, cx| {
                                            ts.last_mouse_pos = None;
                                            cx.notify();
                                        });
                                    }
                                }),
                            )
                            .child(crate::canvas::CanvasElement::new(
                                self.document.clone(),
                                self.tool_state.clone(),
                            )),
                    )
                    .child(
                        div()
                            .w(px(256.))
                            .h_full()
                            .bg(cx.theme().muted)
                            .border_l(px(1.))
                            .border_color(cx.theme().border)
                            .flex()
                            .flex_col()
                            .child(PropertiesPanel {
                                workspace: cx.entity().downgrade(),
                            })
                            .child(
                                div()
                                    .p(px(8.))
                                    .border_t(px(1.))
                                    .border_color(cx.theme().border)
                                    .flex()
                                    .flex_col()
                                    .gap(px(8.))
                                    .child(div().text_size(px(12.)).child("Colors")),
                            )
                            .child(LayerPanel {
                                workspace: cx.entity().downgrade(),
                            }),
                    ),
            )
            .child(
                div()
                    .h(px(24.))
                    .w_full()
                    .bg(cx.theme().muted)
                    .border_t(px(1.))
                    .border_color(cx.theme().border)
                    .px(px(8.))
                    .flex()
                    .items_center()
                    .child(SharedString::from(format!("Tool: {:?}", active_tool))),
            )
            .when_some(self.modal.clone(), |el, modal| el.child(modal))
    }
}
