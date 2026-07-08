use gpui::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct CanvasTransform {
    pub offset: Point<f32>,
    pub scale: f32,
}

impl Default for CanvasTransform {
    fn default() -> Self {
        Self {
            offset: Point::default(),
            scale: 1.0,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct DocumentData {
    pub width: u32,
    pub height: u32,
    pub layers: Vec<LayerData>,
    pub transform: CanvasTransform,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum LayerData {
    Raster {
        name: String,
        visible: bool,
        opacity: f32,
        pixels: Vec<u8>,
    },
}

#[derive(Clone)]
pub enum Action {
    Paint {
        layer_index: usize,
        before_pixels: Vec<u8>,
        after_pixels: Vec<u8>,
    },
    AddLayer {
        index: usize,
        layer_data: LayerData,
    },
    DeleteLayer {
        index: usize,
        layer_data: LayerData,
    },
    MoveLayer {
        from: usize,
        to: usize,
    },
    ToggleVisibility {
        index: usize,
        before: bool,
    },
    SetOpacity {
        index: usize,
        before: f32,
        after: f32,
    },
}

pub struct Document {
    pub size: Size<u32>,
    pub layers: Vec<Entity<Layer>>,
    pub active_layer_index: usize,
    pub transform: CanvasTransform,
    pub undo_stack: Vec<Action>,
    pub redo_stack: Vec<Action>,
}

impl Document {
    pub fn new(size: Size<u32>, cx: &mut App) -> Self {
        let first_layer = cx.new(|_cx| Layer::Raster(RasterLayer::new(size, "Background")));
        Self {
            size,
            layers: vec![first_layer],
            active_layer_index: 0,
            transform: CanvasTransform::default(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn from_data(data: DocumentData, cx: &mut App) -> Self {
        let size = Size {
            width: data.width,
            height: data.height,
        };
        let layers = data
            .layers
            .into_iter()
            .map(|l_data| {
                cx.new(|_cx| match l_data {
                    LayerData::Raster {
                        name,
                        visible,
                        opacity,
                        pixels,
                    } => Layer::Raster(RasterLayer {
                        name,
                        visible,
                        opacity,
                        pixels,
                        render_cache: None,
                    }),
                })
            })
            .collect();

        Self {
            size,
            layers,
            active_layer_index: 0,
            transform: data.transform,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn to_data(&self, cx: &App) -> DocumentData {
        let layers = self
            .layers
            .iter()
            .map(|l_entity| match l_entity.read(cx) {
                Layer::Raster(r) => LayerData::Raster {
                    name: r.name.clone(),
                    visible: r.visible,
                    opacity: r.opacity,
                    pixels: r.pixels.clone(),
                },
            })
            .collect();

        DocumentData {
            width: self.size.width,
            height: self.size.height,
            layers,
            transform: self.transform,
        }
    }

    pub fn add_layer(&mut self, name: &str, cx: &mut App) {
        let new_layer = cx.new(|_cx| Layer::Raster(RasterLayer::new(self.size, name)));
        self.layers.insert(0, new_layer.clone());
        let layer_data = match new_layer.read(cx) {
            Layer::Raster(r) => LayerData::Raster {
                name: r.name.clone(),
                visible: r.visible,
                opacity: r.opacity,
                pixels: r.pixels.clone(),
            },
        };
        self.undo_stack.push(Action::AddLayer {
            index: 0,
            layer_data,
        });
        self.redo_stack.clear();
        self.active_layer_index = 0;
    }

    pub fn delete_layer(&mut self, index: usize, cx: &App) {
        if self.layers.len() > 1 {
            let layer_entity = self.layers.remove(index);
            let layer_data = match layer_entity.read(cx) {
                Layer::Raster(r) => LayerData::Raster {
                    name: r.name.clone(),
                    visible: r.visible,
                    opacity: r.opacity,
                    pixels: r.pixels.clone(),
                },
            };
            self.undo_stack
                .push(Action::DeleteLayer { index, layer_data });
            self.redo_stack.clear();
            if self.active_layer_index >= self.layers.len() {
                self.active_layer_index = self.layers.len() - 1;
            }
        }
    }

    pub fn toggle_visibility(&mut self, index: usize, cx: &mut App) {
        if let Some(layer_entity) = self.layers.get(index) {
            let before = layer_entity.read(cx).visible();
            layer_entity.update(cx, |layer, cx| {
                match layer {
                    Layer::Raster(r) => r.visible = !r.visible,
                }
                cx.notify();
            });
            self.undo_stack.push(Action::ToggleVisibility { index, before });
            self.redo_stack.clear();
        }
    }

    pub fn set_opacity(&mut self, index: usize, opacity: f32, cx: &mut App) {
        if let Some(layer_entity) = self.layers.get(index) {
            let before = layer_entity.read(cx).opacity();
            layer_entity.update(cx, |layer, cx| {
                match layer {
                    Layer::Raster(r) => r.opacity = opacity,
                }
                cx.notify();
            });
            self.undo_stack.push(Action::SetOpacity { index, before, after: opacity });
            self.redo_stack.clear();
        }
    }

    pub fn move_layer(&mut self, from: usize, to: usize) {
        let layer = self.layers.remove(from);
        self.layers.insert(to, layer);
        self.undo_stack.push(Action::MoveLayer { from, to });
        self.redo_stack.clear();
        self.active_layer_index = to;
    }

    pub fn undo(&mut self, cx: &mut App) {
        if let Some(action) = self.undo_stack.pop() {
            match action.clone() {
                Action::Paint {
                    layer_index,
                    before_pixels,
                    ..
                } => {
                    if let Some(layer_entity) = self.layers.get(layer_index) {
                        layer_entity.update(cx, |layer, cx| {
                            let Layer::Raster(raster) = layer;
                            raster.pixels = before_pixels;
                            raster.render_cache = None;
                            cx.notify();
                        });
                    }
                }
                Action::AddLayer { index, .. } => {
                    self.layers.remove(index);
                    if self.active_layer_index >= self.layers.len() {
                        self.active_layer_index = self.layers.len() - 1;
                    }
                }
                Action::DeleteLayer { index, layer_data } => {
                    let layer = cx.new(|_cx| match layer_data {
                        LayerData::Raster {
                            name,
                            visible,
                            opacity,
                            pixels,
                        } => Layer::Raster(RasterLayer {
                            name,
                            visible,
                            opacity,
                            pixels,
                            render_cache: None,
                        }),
                    });
                    self.layers.insert(index, layer);
                    self.active_layer_index = index;
                }
                Action::MoveLayer { from, to } => {
                    let layer = self.layers.remove(to);
                    self.layers.insert(from, layer);
                    self.active_layer_index = from;
                }
                Action::ToggleVisibility { index, before } => {
                    if let Some(layer_entity) = self.layers.get(index) {
                        layer_entity.update(cx, |layer, cx| {
                            match layer {
                                Layer::Raster(r) => r.visible = before,
                            }
                            cx.notify();
                        });
                    }
                }
                Action::SetOpacity { index, before, .. } => {
                    if let Some(layer_entity) = self.layers.get(index) {
                        layer_entity.update(cx, |layer, cx| {
                            match layer {
                                Layer::Raster(r) => r.opacity = before,
                            }
                            cx.notify();
                        });
                    }
                }
            }
            self.redo_stack.push(action);
        }
    }

    pub fn redo(&mut self, cx: &mut App) {
        if let Some(action) = self.redo_stack.pop() {
            match action.clone() {
                Action::Paint {
                    layer_index,
                    after_pixels,
                    ..
                } => {
                    if let Some(layer_entity) = self.layers.get(layer_index) {
                        layer_entity.update(cx, |layer, cx| {
                            let Layer::Raster(raster) = layer;
                            raster.pixels = after_pixels;
                            raster.render_cache = None;
                            cx.notify();
                        });
                    }
                }
                Action::AddLayer { index, layer_data } => {
                    let layer = cx.new(|_cx| match layer_data {
                        LayerData::Raster {
                            name,
                            visible,
                            opacity,
                            pixels,
                        } => Layer::Raster(RasterLayer {
                            name,
                            visible,
                            opacity,
                            pixels,
                            render_cache: None,
                        }),
                    });
                    self.layers.insert(index, layer);
                    self.active_layer_index = index;
                }
                Action::DeleteLayer { index, .. } => {
                    self.layers.remove(index);
                    if self.active_layer_index >= self.layers.len() {
                        self.active_layer_index = self.layers.len() - 1;
                    }
                }
                Action::MoveLayer { from, to } => {
                    let layer = self.layers.remove(from);
                    self.layers.insert(to, layer);
                    self.active_layer_index = to;
                }
                Action::ToggleVisibility { index, before } => {
                    if let Some(layer_entity) = self.layers.get(index) {
                        layer_entity.update(cx, |layer, cx| {
                            match layer {
                                Layer::Raster(r) => r.visible = !before,
                            }
                            cx.notify();
                        });
                    }
                }
                Action::SetOpacity { index, after, .. } => {
                    if let Some(layer_entity) = self.layers.get(index) {
                        layer_entity.update(cx, |layer, cx| {
                            match layer {
                                Layer::Raster(r) => r.opacity = after,
                            }
                            cx.notify();
                        });
                    }
                }
            }
            self.undo_stack.push(action);
        }
    }

    pub fn active_layer(&self) -> Option<&Entity<Layer>> {
        self.layers.get(self.active_layer_index)
    }
}

pub enum Layer {
    Raster(RasterLayer),
}

impl Layer {
    pub fn pixels(&self) -> &Vec<u8> {
        match self {
            Layer::Raster(r) => &r.pixels,
        }
    }

    pub fn visible(&self) -> bool {
        match self {
            Layer::Raster(r) => r.visible,
        }
    }

    pub fn opacity(&self) -> f32 {
        match self {
            Layer::Raster(r) => r.opacity,
        }
    }
}

pub struct RasterLayer {
    pub name: String,
    pub visible: bool,
    pub opacity: f32,
    pub pixels: Vec<u8>, // BGRA (Matches GPUI expectation)
    pub render_cache: Option<Arc<RenderImage>>,
}

impl RasterLayer {
    pub fn new(size: Size<u32>, name: &str) -> Self {
        let pixel_count = (size.width * size.height) as usize;
        Self {
            name: name.to_string(),
            visible: true,
            opacity: 1.0,
            pixels: vec![0; pixel_count * 4],
            render_cache: None,
        }
    }
}
