use gpui::*;
use gpui::prelude::FluentBuilder;

use crate::document::{Document, DocumentData, Layer, LayerData, RasterLayer};
use crate::tool::{ToolState, Tool, ToolEvent};
use crate::canvas::CanvasElement;
use gpui_component::color_picker::{ColorPicker, ColorPickerEvent, ColorPickerState};
use gpui_component::ActiveTheme;
use std::sync::Arc;

actions!(sienna, [Undo, Redo, NewProject]);

pub struct PixelInput {
    pub value: String,
    focus_handle: FocusHandle,
}

impl PixelInput {
    pub fn new(initial: u32, cx: &mut Context<Self>) -> Self {
        Self {
            value: initial.to_string(),
            focus_handle: cx.focus_handle(),
        }
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let key = event.keystroke.key.as_str();
        if key == "backspace" {
            self.value.pop();
            cx.emit(InputEvent::Changed);
            cx.notify();
        } else if key == "enter" {
            cx.notify();
        } else if key.len() == 1 && key.chars().next().unwrap().is_ascii_digit() {
            if self.value.len() < 5 {
                self.value.push_str(key);
                cx.emit(InputEvent::Changed);
                cx.notify();
            }
        }
    }
}

pub enum InputEvent {
    Changed,
}

impl EventEmitter<InputEvent> for PixelInput {}

impl Render for PixelInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_focused = self.focus_handle.is_focused(window);
        
        div()
            .id("pixel-input")
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::on_key_down))
            .on_mouse_down(MouseButton::Left, {
                let focus_handle = self.focus_handle.clone();
                move |_, window, cx| {
                    focus_handle.focus(window, cx);
                }
            })
            .px(px(4.))
            .w(px(64.))
            .bg(if is_focused { cx.theme().muted } else { cx.theme().border })
            .border(px(1.))
            .border_color(if is_focused { cx.theme().ring } else { cx.theme().border })
            .rounded(px(2.))
            .text_size(px(12.))
            .flex()
            .items_center()
            .child(self.value.clone())
            .child(if is_focused {
                div().w(px(1.)).h(px(12.)).bg(cx.theme().primary_foreground)
            } else {
                div()
            })
    }
}

pub struct NewProjectModal {
    width_input: Entity<PixelInput>,
    height_input: Entity<PixelInput>,
    on_create: Arc<dyn Fn(u32, u32, &mut Window, &mut App) + 'static>,
    on_cancel: Arc<dyn Fn(&mut Window, &mut App) + 'static>,
}

impl NewProjectModal {
    pub fn new(
        cx: &mut Context<Self>,
        on_create: impl Fn(u32, u32, &mut Window, &mut App) + 'static,
        on_cancel: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            width_input: cx.new(|cx| PixelInput::new(1024, cx)),
            height_input: cx.new(|cx| PixelInput::new(768, cx)),
            on_create: Arc::new(on_create),
            on_cancel: Arc::new(on_cancel),
        }
    }
}

impl Render for NewProjectModal {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .absolute()
            .size_full()
            .bg(rgba(0x000000aa))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w(px(300.))
                    .p(px(16.))
                    .bg(cx.theme().muted)
                    .rounded(px(8.))
                    .shadow_md()
                    .flex()
                    .flex_col()
                    .gap(px(16.))
                    .child(div().text_size(px(18.)).font_weight(FontWeight::BOLD).child("New Project"))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(8.))
                            .child(
                                div()
                                    .flex()
                                    .justify_between()
                                    .items_center()
                                    .child("Width (px)")
                                    .child(self.width_input.clone())
                            )
                            .child(
                                div()
                                    .flex()
                                    .justify_between()
                                    .items_center()
                                    .child("Height (px)")
                                    .child(self.height_input.clone())
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .gap(px(8.))
                            .child(
                                div()
                                    .id("cancel-btn")
                                    .px(px(12.))
                                    .py(px(6.))
                                    .bg(cx.theme().border)
                                    .rounded(px(4.))
                                    .hover(|s| s.bg(cx.theme().muted))
                                    .on_click({
                                        let on_cancel = self.on_cancel.clone();
                                        move |_, window, cx| (on_cancel)(window, cx)
                                    })
                                    .child("Cancel")
                            )
                            .child(
                                div()
                                    .id("create-btn")
                                    .px(px(12.))
                                    .py(px(6.))
                                    .bg(cx.theme().primary)
                                    .text_color(cx.theme().primary_foreground)
                                    .rounded(px(4.))
                                    .hover(|s| s.bg(cx.theme().primary))
                                    .on_click({
                                        let width_input = self.width_input.clone();
                                        let height_input = self.height_input.clone();
                                        let on_create = self.on_create.clone();
                                        move |_, window, cx| {
                                            let w = width_input.read(cx).value.parse::<u32>().unwrap_or(1024);
                                            let h = height_input.read(cx).value.parse::<u32>().unwrap_or(768);
                                            (on_create)(w, h, window, cx);
                                        }
                                    })
                                    .child("Create")
                            )
                    )
            )
    }
}

pub struct Workspace {
    document: Entity<Document>,
    tool_state: Entity<ToolState>,
    modal: Option<Entity<NewProjectModal>>,
    focus_handle: FocusHandle,
    color_picker_state: Entity<ColorPickerState>,
    canvas_hitbox: Option<Hitbox>,
}

impl Workspace {
    pub fn new(document: Entity<Document>, tool_state: Entity<ToolState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let window_handle = window.window_handle();
        let initial_color = tool_state.read(cx).active_color;
        let color_picker_state = cx.new(|cx| {
            ColorPickerState::new(window, cx)
                .default_value(Hsla::from(initial_color))
        });
        
        cx.subscribe(&color_picker_state, |this, _entity, event, cx| {
            let ColorPickerEvent::Change(color) = event;
            if let Some(color) = color {
                let color_rgba = Rgba::from(*color);
                this.tool_state.update(cx, |ts, cx| {
                    if ts.active_color != color_rgba {
                        ts.active_color = color_rgba;
                        cx.emit(ToolEvent::ColorChanged(color_rgba));
                        cx.notify();
                    }
                });
                cx.notify();
            }
        }).detach();

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
        }).detach();

        cx.observe(&tool_state, |_this, _entity, cx| {
            cx.notify();
        }).detach();

        cx.observe(&document, |_this, _entity, cx| {
            cx.notify();
        }).detach();

        Self {
            document,
            tool_state,
            modal: None,
            focus_handle: cx.focus_handle(),
            color_picker_state,
            canvas_hitbox: None,
        }
    }
    pub fn set_canvas_hitbox(&mut self, hitbox: Hitbox, cx: &mut Context<Self>) {
        self.canvas_hitbox = Some(hitbox);
        cx.notify();
    }

    fn select_tool(&mut self, tool: Tool, cx: &mut Context<Self>) {
        self.tool_state.update(cx, |state, cx| {
            state.active_tool = tool;
            cx.notify();
        });
        cx.notify();
    }

    fn new_project(&mut self, _: &NewProject, _window: &mut Window, cx: &mut Context<Self>) {
        let workspace = cx.entity().downgrade();
        let modal = cx.new(|cx| {
            let ws_create = workspace.clone();
            let ws_cancel = workspace.clone();
            NewProjectModal::new(
                cx,
                move |w, h, _window, cx| {
                    let _ = ws_create.update(cx, |this, cx| {
                        this.document.update(cx, |doc, cx| {
                            *doc = Document::new(Size { width: w, height: h }, cx);
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
            let LayerData::Raster { pixels, .. } = layer;
            for chunk in pixels.chunks_exact_mut(4) {
                chunk.swap(0, 2);
            }
        }
        cx.spawn(|_this, cx: &mut AsyncApp| {
            let cx = cx.clone();
            async move {
                let file = rfd::AsyncFileDialog::new().add_filter("Sienna", &["sienna"]).save_file().await;
                if let Some(file) = file {
                    let path = file.path().to_path_buf();
                    cx.background_spawn(async move {
                        let encoded = bincode::serialize(&doc_data).unwrap();
                        std::fs::write(path, encoded).unwrap();
                    }).await;
                }
            }
        }).detach();
    }

    fn open(&mut self, _event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let document_entity = self.document.downgrade();
        cx.spawn(|_this, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                let file = rfd::AsyncFileDialog::new().add_filter("Sienna", &["sienna"]).pick_file().await;
                if let Some(file) = file {
                    let path = file.path().to_path_buf();
                    let mut data = cx.background_spawn(async move {
                        let bytes = std::fs::read(path).unwrap();
                        let data: DocumentData = bincode::deserialize(&bytes).unwrap();
                        data
                    }).await;

                    for layer in &mut data.layers {
                        let LayerData::Raster { pixels, .. } = layer;
                        for chunk in pixels.chunks_exact_mut(4) {
                            chunk.swap(0, 2);
                        }
                    }

                    document_entity.update(&mut cx, |doc, cx| {
                        *doc = Document::from_data(data, cx);
                        cx.notify();
                    }).ok();
                }
            }
        }).detach();
    }

    fn import_image(&mut self, _event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let document_entity = self.document.downgrade();
        cx.spawn(|_this, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                let file = rfd::AsyncFileDialog::new().add_filter("Images", &["png", "jpg", "jpeg", "webp"]).pick_file().await;
                if let Some(file) = file {
                    let path = file.path().to_path_buf();
                    let layer_name = file.file_name();
                    let mut layer_data = cx.background_spawn(async move {
                        let img = image::open(path).expect("Failed to open image").to_rgba8();
                        LayerData::Raster { name: layer_name, visible: true, opacity: 1.0, pixels: img.into_raw() }
                    }).await;

                    let LayerData::Raster { pixels, .. } = &mut layer_data;
                    for chunk in pixels.chunks_exact_mut(4) {
                        chunk.swap(0, 2);
                    }

                    document_entity.update(&mut cx, |doc, cx| {
                        let layer = cx.new(|_cx| match layer_data {
                            LayerData::Raster { name, visible, opacity, pixels } => {
                                Layer::Raster(RasterLayer { name, visible, opacity, pixels, render_cache: None })
                            }
                        });
                        doc.layers.push(layer);
                        doc.active_layer_index = doc.layers.len() - 1;
                        cx.notify();
                    }).ok();
                }
            }
        }).detach();
    }

    fn add_layer(&mut self, _event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.document.update(cx, |doc, cx| {
            let name = format!("Layer {}", doc.layers.len() + 1);
            doc.add_layer(&name, cx);
            cx.notify();
        });
    }

    fn delete_layer(&mut self, _event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.document.update(cx, |doc, cx| {
            doc.delete_layer(doc.active_layer_index, cx);
            cx.notify();
        });
    }

    fn move_layer_up(&mut self, _event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.document.update(cx, |doc, cx| {
            let idx = doc.active_layer_index;
            if idx > 0 { doc.move_layer(idx, idx - 1); }
            cx.notify();
        });
    }

    fn move_layer_down(&mut self, _event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.document.update(cx, |doc, cx| {
            let idx = doc.active_layer_index;
            if idx < doc.layers.len() - 1 { doc.move_layer(idx, idx + 1); }
            cx.notify();
        });
    }

    fn undo(&mut self, _: &Undo, _window: &mut Window, cx: &mut Context<Self>) {
        self.document.update(cx, |doc, cx| { doc.undo(cx); cx.notify(); });
    }

    fn redo(&mut self, _: &Redo, _window: &mut Window, cx: &mut Context<Self>) {
        self.document.update(cx, |doc, cx| { doc.redo(cx); cx.notify(); });
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let active_tool = self.tool_state.read(cx).active_tool;
        let doc = self.document.read(cx);
        let layers = doc.layers.clone();
        let active_layer_index = doc.active_layer_index;
        let brush_size = self.tool_state.read(cx).brush_size;
        let brush_opacity = self.tool_state.read(cx).brush_opacity;

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
                    .child(div().text_size(px(14.)).font_weight(FontWeight::BOLD).child("SIENNA"))
                    .child(menu_button("new-btn", "New", cx.listener(|this, _, window, cx| this.new_project(&NewProject, window, cx)), cx))
                    .child(menu_button("open-btn", "Open", cx.listener(Self::open), cx))
                    .child(menu_button("save-btn", "Save", cx.listener(Self::save), cx))
                    .child(menu_button("import-btn", "Import", cx.listener(Self::import_image), cx))
                    .child(div().w(px(12.)))
                    .child(menu_button("undo-btn", "Undo", cx.listener(|this, _, window, cx| this.undo(&Undo, window, cx)), cx))
                    .child(menu_button("redo-btn", "Redo", cx.listener(|this, _, window, cx| this.redo(&Redo, window, cx)), cx))
            )
            .child(
                div()
                    .flex_grow()
                    .flex()
                    .child(
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
                            .child(tool_button("M", "move-tool", active_tool == Tool::Move, cx.listener(move |this, _, _, cx| this.select_tool(Tool::Move, cx)), cx))
                            .child(tool_button("B", "brush-tool", active_tool == Tool::Brush, cx.listener(move |this, _, _, cx| this.select_tool(Tool::Brush, cx)), cx))
                            .child(tool_button("E", "eraser-tool", active_tool == Tool::Eraser, cx.listener(move |this, _, _, cx| this.select_tool(Tool::Eraser, cx)), cx))
                            .child(tool_button("P", "picker-tool", active_tool == Tool::ColorPicker, cx.listener(move |this, _, _, cx| this.select_tool(Tool::ColorPicker, cx)), cx))
                            .child(div().w_full().h(px(1.)).bg(cx.theme().border))
                            .child(div().id("sidebar-color-picker").child(ColorPicker::new(&self.color_picker_state)))
                    )
                    .child(
                        div()
                            .flex_grow()
                            .h_full()
                            .on_scroll_wheel(cx.listener(|this, event: &ScrollWheelEvent, _window, cx| {
                                let (document, canvas_hitbox) = (this.document.clone(), this.canvas_hitbox.clone());
                                if let Some(canvas_hitbox) = canvas_hitbox {
                                    if canvas_hitbox.is_hovered(_window) {
                                        document.update(cx, |doc, cx| {
                                            let delta = match event.delta {
                                                ScrollDelta::Pixels(p) => p,
                                                ScrollDelta::Lines(l) => l.map(|v| px(v * 20.0)),
                                            };

                                            if event.modifiers.secondary() {
                                                let factor = if delta.y.to_f64() > 0.0 { 1.1 } else { 0.9 };
                                                doc.transform.scale *= factor;
                                                doc.transform.scale = doc.transform.scale.clamp(0.01, 100.0);
                                            } else {
                                                doc.transform.offset.x += delta.x.to_f64() as f32;
                                                doc.transform.offset.y += delta.y.to_f64() as f32;
                                            }
                                            cx.notify();
                                        });
                                    }
                                }
                            }))
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, event: &MouseDownEvent, _window, cx| {
                                let (document, tool_state, canvas_hitbox) = (this.document.clone(), this.tool_state.clone(), this.canvas_hitbox.clone());
                                if let Some(canvas_hitbox) = canvas_hitbox {
                                    if canvas_hitbox.is_hovered(_window) {
                                        let tool = tool_state.read(cx).active_tool;
                                        if tool == Tool::Brush || tool == Tool::Eraser {
                                            document.update(cx, |doc, cx| {
                                                if let Some(active_layer) = doc.active_layer() {
                                                    let pixels_before = active_layer.read(cx).pixels().clone();
                                                    tool_state.update(cx, |ts, cx| {
                                                        ts.pixels_before = Some(pixels_before);
                                                        ts.last_mouse_pos = Some(event.position);
                                                        cx.notify();
                                                    });
                                                }
                                            });
                                        } else if tool == Tool::Move {
                                            tool_state.update(cx, |ts, cx| {
                                                ts.last_mouse_pos = Some(event.position);
                                                cx.notify();
                                            });
                                        } else if tool == Tool::ColorPicker {
                                            let doc_size = document.read(cx).size;
                                            let transform = document.read(cx).transform;
                                            let bounds = canvas_hitbox.bounds;
                                            
                                            let screen_pos = event.position - bounds.origin;
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
                                                    let Layer::Raster(raster) = layer;
                                                    if raster.visible {
                                                        let idx = ((y * doc_size.width as i32 + x) * 4) as usize;
                                                        let a = raster.pixels[idx+3];
                                                        if a > 0 {
                                                            picked_color = Some(Rgba {
                                                                r: raster.pixels[idx+2] as f32 / 255.0,
                                                                g: raster.pixels[idx+1] as f32 / 255.0,
                                                                b: raster.pixels[idx] as f32 / 255.0,
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
                                }
                            }))
                            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _window, cx| {
                                let (document, tool_state, canvas_hitbox) = (this.document.clone(), this.tool_state.clone(), this.canvas_hitbox.clone());
                                if let Some(canvas_hitbox) = canvas_hitbox {
                                    // For move/brush we might want to continue even if mouse leaves the div, 
                                    // but for now we'll stick to the div boundaries for simplicity.
                                    let (active_tool, brush_size, brush_opacity, active_color) = {
                                        let ts = tool_state.read(cx);
                                        (ts.active_tool, ts.brush_size, ts.brush_opacity, ts.active_color)
                                    };

                                    let transform = document.read(cx).transform;
                                    let bounds = canvas_hitbox.bounds;
                                    let origin = bounds.origin;
                                    let doc_size = document.read(cx).size;

                                    if active_tool == Tool::Move && event.pressed_button == Some(MouseButton::Left) {
                                        if let Some(last_pos) = tool_state.read(cx).last_mouse_pos {
                                            let delta = event.position - last_pos;
                                            document.update(cx, |doc, cx| {
                                                doc.transform.offset.x += delta.x.to_f64() as f32;
                                                doc.transform.offset.y += delta.y.to_f64() as f32;
                                                cx.notify();
                                            });
                                        }
                                        tool_state.update(cx, |ts, cx| {
                                            ts.last_mouse_pos = Some(event.position);
                                            cx.notify();
                                        });
                                    } else if (active_tool == Tool::Brush || active_tool == Tool::Eraser) && event.pressed_button == Some(MouseButton::Left) {
                                        let screen_pos = event.position - origin;
                                        let canvas_pos = Point {
                                            x: (screen_pos.x.to_f64() as f32 - transform.offset.x) / transform.scale,
                                            y: (screen_pos.y.to_f64() as f32 - transform.offset.y) / transform.scale,
                                        };
                                        
                                        let last_pos = tool_state.read(cx).last_mouse_pos.map(|p| {
                                            let p_rel = p - origin;
                                            Point {
                                                x: (p_rel.x.to_f64() as f32 - transform.offset.x) / transform.scale,
                                                y: (p_rel.y.to_f64() as f32 - transform.offset.y) / transform.scale,
                                            }
                                        });

                                        let active_layer_entity = document.read(cx).active_layer().cloned();

                                        if let Some(active_layer_entity) = active_layer_entity {
                                            let mut pixels_snapshot: Option<Vec<u8>> = None;
                                            let doc_width = doc_size.width;
                                            let doc_height = doc_size.height;
                                            
                                            active_layer_entity.update(cx, |layer, cx| {
                                                let Layer::Raster(raster) = layer;
                                                let mut changed = false;

                                                let points = if let Some(last) = last_pos {
                                                    let dx = canvas_pos.x - last.x;
                                                    let dy = canvas_pos.y - last.y;
                                                    let dist = (dx * dx + dy * dy).sqrt();
                                                    let steps = (dist / (brush_size / 4.0)).max(1.0) as usize;
                                                    (0..=steps).map(|i| {
                                                        let t = i as f32 / steps as f32;
                                                        Point { x: last.x + dx * t, y: last.y + dy * t }
                                                    }).collect::<Vec<_>>()
                                                } else {
                                                    vec![canvas_pos]
                                                };

                                                let b = (active_color.b * 255.0) as u32;
                                                let g = (active_color.g * 255.0) as u32;
                                                let r = (active_color.r * 255.0) as u32;
                                                let a_brush = (brush_opacity * 255.0) as u32;

                                                for p in points {
                                                    let x = p.x as i32;
                                                    let y = p.y as i32;
                                                    let radius = (brush_size / 2.0) as i32;
                                                    for dy in -radius..radius {
                                                        for dx in -radius..radius {
                                                            if dx*dx + dy*dy <= radius*radius {
                                                                let px = x + dx;
                                                                let py = y + dy;
                                                                if px >= 0 && px < doc_width as i32 && py >= 0 && py < doc_height as i32 {
                                                                    let idx = ((py * doc_width as i32 + px) * 4) as usize;
                                                                    if active_tool == Tool::Eraser {
                                                                        if a_brush == 255 {
                                                                            raster.pixels[idx] = 0;
                                                                            raster.pixels[idx+1] = 0;
                                                                            raster.pixels[idx+2] = 0;
                                                                            raster.pixels[idx+3] = 0;
                                                                        } else {
                                                                            let current_a = raster.pixels[idx+3] as u32;
                                                                            let remove_a = a_brush;
                                                                            let new_a = current_a.saturating_sub(remove_a);
                                                                            if new_a == 0 {
                                                                                raster.pixels[idx] = 0;
                                                                                raster.pixels[idx+1] = 0;
                                                                                raster.pixels[idx+2] = 0;
                                                                                raster.pixels[idx+3] = 0;
                                                                            } else {
                                                                                raster.pixels[idx+3] = new_a as u8;
                                                                            }
                                                                        }
                                                                        changed = true;
                                                                    } else {
                                                                        if a_brush == 255 {
                                                                            raster.pixels[idx] = b as u8;
                                                                            raster.pixels[idx+1] = g as u8;
                                                                            raster.pixels[idx+2] = r as u8;
                                                                            raster.pixels[idx+3] = 255;
                                                                        } else {
                                                                            let a_src = a_brush;
                                                                            let a_dst = raster.pixels[idx+3] as u32;
                                                                            let inv_a = 255 - a_src;
                                                                            raster.pixels[idx] = ((b * a_src + raster.pixels[idx] as u32 * inv_a) / 255) as u8;
                                                                            raster.pixels[idx+1] = ((g * a_src + raster.pixels[idx+1] as u32 * inv_a) / 255) as u8;
                                                                            raster.pixels[idx+2] = ((r * a_src + raster.pixels[idx+2] as u32 * inv_a) / 255) as u8;
                                                                            raster.pixels[idx+3] = (a_src + (a_dst * inv_a) / 255) as u8;
                                                                        }
                                                                        changed = true;
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }

                                                if changed {
                                                    pixels_snapshot = Some(raster.pixels.clone());
                                                    cx.notify();
                                                    document.update(cx, |_, cx| cx.notify());
                                                }
                                            });

                                            if let Some(pixels) = pixels_snapshot {
                                                let layer_entity = active_layer_entity.clone();
                                                cx.spawn(move |_, cx: &mut AsyncApp| {
                                                    let mut cx = cx.clone();
                                                    async move {
                                                        let render_image = cx.background_spawn(async move {
                                                            let buffer = image::RgbaImage::from_raw(doc_width, doc_height, pixels).unwrap();
                                                            let frame = image::Frame::new(buffer);
                                                            Arc::new(RenderImage::new(smallvec::smallvec![frame]))
                                                        }).await;
                                                        
                                                        let _ = layer_entity.update(&mut cx, |layer, cx| {
                                                            let Layer::Raster(raster) = layer;
                                                            raster.render_cache = Some(render_image);
                                                            cx.notify();
                                                        });
                                                    }
                                                }).detach();
                                            }
                                        }

                                        tool_state.update(cx, |ts, cx| {
                                            ts.last_mouse_pos = Some(event.position);
                                            cx.notify();
                                        });
                                    }
                                }
                            }))
                            .on_mouse_up(MouseButton::Left, cx.listener(|this, event: &MouseUpEvent, _window, cx| {
                                let (document, tool_state) = (this.document.clone(), this.tool_state.clone());
                                let tool = tool_state.read(cx).active_tool;
                                if tool == Tool::Brush || tool == Tool::Eraser {
                                    let pixels_before = tool_state.read(cx).pixels_before.clone();
                                    if let Some(before) = pixels_before {
                                        document.update(cx, |doc, cx| {
                                            if let Some(active_layer) = doc.active_layer() {
                                                let pixels_after = active_layer.read(cx).pixels().clone();
                                                if before != pixels_after {
                                                    doc.undo_stack.push(crate::document::Action::Paint {
                                                        layer_index: doc.active_layer_index,
                                                        before_pixels: before,
                                                        after_pixels: pixels_after,
                                                    });
                                                    doc.redo_stack.clear();
                                                }
                                            }
                                        });
                                    }
                                    tool_state.update(cx, |ts, cx| {
                                        ts.pixels_before = None;
                                        ts.last_mouse_pos = None;
                                        cx.notify();
                                    });
                                } else if tool == Tool::Move {
                                    tool_state.update(cx, |ts, cx| {
                                        ts.last_mouse_pos = None;
                                        cx.notify();
                                    });
                                }
                            }))
                            .child(CanvasElement::new(cx.entity().downgrade(), self.document.clone(), self.tool_state.clone()))
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
                            .child(div().p(px(8.)).border_b(px(1.)).border_color(cx.theme().border).child("Properties"))
                            .child(
                                div()
                                    .p(px(8.))
                                    .flex()
                                    .flex_col()
                                    .gap(px(8.))
                                    .child(property_row("Size", format!("{:.0}", brush_size), cx.listener(|this, _, _, cx| {
                                        this.tool_state.update(cx, |ts, cx| { ts.brush_size = if ts.brush_size >= 100.0 { 5.0 } else { ts.brush_size + 5.0 }; cx.notify(); });
                                    }), cx))
                                    .child(property_row("Opacity", format!("{:.0}%", brush_opacity * 100.0), cx.listener(|this, _, _, cx| {
                                        this.tool_state.update(cx, |ts, cx| { ts.brush_opacity = if ts.brush_opacity <= 0.1 { 1.0 } else { ts.brush_opacity - 0.1 }; cx.notify(); });
                                    }), cx))
                            )
                            .child(
                                div()
                                    .p(px(8.))
                                    .border_t(px(1.))
                                    .border_color(cx.theme().border)
                                    .flex()
                                    .flex_col()
                                    .gap(px(8.))
                                    .child(div().text_size(px(12.)).child("Colors"))
                            )
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
                                            .child(icon_button("layer-add", "+", cx.listener(Self::add_layer), cx))
                                            .child(icon_button("layer-del", "-", cx.listener(Self::delete_layer), cx))
                                            .child(icon_button("layer-up", "↑", cx.listener(Self::move_layer_up), cx))
                                            .child(icon_button("layer-down", "↓", cx.listener(Self::move_layer_down), cx))
                                    )
                            )
                            .child(
                                div()
                                    .flex_grow()
                                    .children(layers.into_iter().enumerate().rev().map(|(idx, layer_entity)| {
                                        let is_active = idx == active_layer_index;
                                        let name = match layer_entity.read(cx) { Layer::Raster(r) => r.name.clone() };
                                        div()
                                            .id(("layer", idx))
                                            .p(px(8.))
                                            .bg(if is_active { cx.theme().border } else { cx.theme().muted })
                                            .hover(|s| s.bg(cx.theme().accent))
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.document.update(cx, |doc, cx| { doc.active_layer_index = idx; cx.notify(); });
                                                cx.notify();
                                            }))
                                            .child(name)
                                    }))
                            )
                    )
            )
            .child(div().h(px(24.)).w_full().bg(cx.theme().muted).border_t(px(1.)).border_color(cx.theme().border).px(px(8.)).flex().items_center().child(format!("Tool: {:?}", active_tool)))
            .when_some(self.modal.clone(), |el, modal| el.child(modal))
    }
}

fn tool_button(label: &'static str, id: &'static str, active: bool, on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static, cx: &App) -> impl IntoElement {
    div().id(id).size(px(32.)).flex().items_center().justify_center().rounded(px(4.)).bg(if active { cx.theme().accent } else { cx.theme().muted }).hover(|s| s.bg(cx.theme().border)).on_click(on_click).child(label)
}

fn menu_button(id: &'static str, label: &'static str, on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static, cx: &App) -> impl IntoElement {
    div().id(id).px(px(8.)).py(px(4.)).rounded(px(4.)).text_size(px(13.)).hover(|s| s.bg(cx.theme().border)).on_click(on_click).child(label)
}

fn icon_button(id: &'static str, label: &'static str, on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static, cx: &App) -> impl IntoElement {
    div().id(id).size(px(20.)).flex().items_center().justify_center().rounded(px(2.)).bg(cx.theme().border).hover(|s| s.bg(cx.theme().muted)).on_click(on_click).child(label)
}

fn property_row(label: &'static str, value: String, on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static, cx: &App) -> impl IntoElement {
    div().flex().justify_between().items_center().child(div().text_size(px(12.)).child(label)).child(div().id(SharedString::from(format!("prop-{}", label))).px(px(4.)).bg(cx.theme().border).rounded(px(2.)).text_size(px(12.)).hover(|s| s.bg(cx.theme().muted)).on_click(on_click).child(value))
}

